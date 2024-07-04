<script setup lang="ts">
import _ from 'lodash'

import { open } from '@tauri-apps/api/dialog';

import { ref, reactive, onMounted, watch } from 'vue'
import EULA_game from '../assets/eula.txt?raw'
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api';

import jobsVue from '../components/jobs.vue'
import wicJobs from '../lib/wic-jobs';

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg'
import iconTriangleExclamation from '@fortawesome/fontawesome-free/svgs/solid/triangle-exclamation.svg'

const manager = wicJobs.installManager
manager.clearJobs();
const progress = wicJobs.progress

const DEFAULT_INSTALL_DIR = 'C:\\Program Files (x86)\\Ubisoft\\World in Conflict'

const _installDir = ref(localStorage.getItem('install-dir') || DEFAULT_INSTALL_DIR)
const _step = ref('eula')
const _done = ref(false)

const _jobs = manager.getJobs()

// let path_zipped = '';
// let path_unzipped = '';
// let path_patch10 = '';
// let path_patch11 = '';
// let path_vcredist = '';

let path_vcredist = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_14.exe';

let path_zipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\wic.zip'


let jobs = {
  download_vcredist: async job => {
    const progressId = progress.on({ type: 'download-vcredist' }, (progress) => {
      job.progress = progress.percentage
    })
    path_vcredist = await invoke('download_vcredist');
    progress.off(progressId)
  },
  install_vcredist: async job => {
    try {
      await invoke('install_vcredist', { vcredistExe: path_vcredist });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },

  download_game: async job => {
    const progressId = progress.on({ type: 'download-game' }, (progress) => {
      job.progress = progress.percentage
    })
    path_zipped = await invoke('download_game');
    progress.off(progressId)
  },
  install_game: async job => {
    try {
      const progressId = progress.on({ type: 'install-game' }, (progress) => {
        job.progress = progress.percentage
      })
      await invoke('install_game', { target: _installDir.value, zip: path_zipped });
      progress.off(progressId)
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  create_document_directory: async job => {
    try {
      await invoke('create_document_directory');
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  write_registry_keys: async job => {
    try {
      await invoke('write_registry_keys', { installPath: _installDir.value });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },

  enable_environment: async job => {
    try {
      invoke('environment_set', { environment: "testing" });
      localStorage.setItem('environment', "testing");
      invoke('disable_patches')
      localStorage.setItem('patches-enabled', "false")
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  }
}

const goes = async () => {
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('not elevated, setting install dir and elevating permissions', _installDir.value)
    localStorage.setItem('do-install', _installDir.value);
    await invoke('elevate_permissions')
    return;
  }

  console.log('elevated, continuing installation', localStorage.getItem('do-install'))
  console.log('install dir', _installDir.value)
  localStorage.removeItem('do-install')

  const todo: [string, Function][] = []

  const isInstalled = await invoke('get_install_path')
  console.log("INSTALLED", isInstalled)
  if (!isInstalled) {
    todo.push(["Download Visual Studio C++ Redistributable", jobs.download_vcredist])
    todo.push(["Install Visual Studio C++ Redistributable", jobs.install_vcredist])
    todo.push(["Download game", jobs.download_game])
    todo.push(["Install Game", jobs.install_game])
    todo.push(["Create Document Directory", jobs.create_document_directory])
    todo.push(["Write registry keys", jobs.write_registry_keys])
    todo.push(["Enable testing environment", jobs.enable_environment])
  }

  for (let job of todo) {
    await manager.runJob(job[0], job[1])
  }

  _done.value = true;
}

watch(_installDir, (val) => {
  console.log('SETTING INSTALL DIR', val)
  localStorage.setItem('install-dir', val)
})

onMounted(async () => {
  const route = useRoute()
  if (route.params.step == 'goes') {
    _step.value = 'goes'
    goes()
  }
})

const selectInstallDir = async () => {
  const selected = await open({
    multiple: false,
    directory: true,
    defaultPath: _installDir.value,
  });
  if (selected === null) {
    // user cancelled the selection
  } else {
    _installDir.value = selected as string
  }
}
</script>

<template>
  <div class="card" id="install">
    <div class="card-header">
      Install World in Conflict
    </div>
    <div class="card-body" v-if="_step == 'eula'">
      <div id="eula">
        {{ EULA_game }}
      </div>
      <button @click="_step = 'location'" class="cta">Accept License Agreement</button>
    </div>
    <div class="card-body" v-if="_step == 'location'">
      <div class="mb-3">
        <label for="install-location" class="form-label">Select install location</label>
        <input type="text" class="form-control" id="install-location" v-model="_installDir" @click="selectInstallDir">
      </div>
      <button @click="_step = 'goes'; goes()" class="cta">Download and install</button>
    </div>
    <div v-if="_step == 'goes'" class="card-body">
      <p style="display:block">Installing to {{ _installDir }}.</p>
      <jobs-vue :jobs="_jobs" id="install-jobs" />
    </div>
    <div id="post-install" v-if="_done">
      <div id="post-install-content">
        <div class="alert alert-success done">
          World in Conflict installed successfully
        </div>
        <div class="alert alert-danger done" v-if="_done">
          <iconTriangleExclamation class="icon" />
          If you're getting any errors when starting World in Conflict, reboot and try again!
          <iconTriangleExclamation class="icon" />
        </div>
        <router-link to="/" class="cta secondary">Configure World in Conflict</router-link>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
#install {
  .done {
    border-radius: 0;

    a:first-of-type {
      margin-bottom: 15px;
    }
  }
}

#eula {
  white-space: pre-line;
  height: 350px;
  overflow-y: scroll;
}

#post-install {
  position: absolute;
  top: 10%;
  left: 10%;
  width: 80vw;
  height: 80vh;
  margin: 0;
  padding: 0;
  background: rgba(0, 0, 0, 0.74);
  border-radius: 10px;
}

#post-install-content {
  position: absolute;
  width: 80%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: white;
  text-align: center;

  .alert-success {
    background: rgb(17, 185, 101);
  }

  .alert-danger {
    font-size: 1.5em;
    background: rgb(199, 3, 3);
  }

  .icon {
    height: 1em;
    fill: white;
    padding-bottom: 5px;
  }
}
</style>