<script setup lang="ts">
import _ from 'lodash'

import { ref, reactive, onMounted } from 'vue'
import EULA_game from '../../assets/eula.txt?raw'
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api';

import jobsVue from '../jobs.vue'
import wicJobs from '../../lib/wic-jobs';

const manager = wicJobs.manager
manager.clearJobs();
const progress = wicJobs.progress

const _installDir = ref('C:\\Program Files (x86)\\Sierra Entertainment\\World in Conflict')
const _step = ref('eula')
const _done = ref(false)

const _jobs = wicJobs._jobs

// let path_zipped = '';
// let path_unzipped = '';
// let path_patch10 = '';
// let path_patch11 = '';
// let path_vcredist = '';

let path_zipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en.zip'
let path_unzipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en'
let path_patch10 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
let path_patch11 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
let path_vcredist11 = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_11.exe';
let path_vcredist14 = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_14.exe';


let jobs = {
  download_game: async job => {
    const progressId = progress.on({ type: 'download-game' }, (progress) => {
      job.progress = progress.percentage
    })
    path_zipped = await invoke('download_game');
    progress.off(progressId)
  },
  unzip_game: async job => {
    const progressId = progress.on({ type: 'extract-game' }, (progress) => {
      job.progress = progress.percentage
    })
    path_unzipped = await invoke('unzip_game', { zipPath: path_zipped });
    progress.off(progressId)
  },
  download_patch10: async job => {
    const progressId = progress.on({ type: 'download-patch' }, (progress) => {
      job.progress = progress.percentage
    })
    path_patch10 = await invoke('download_patch', { patch: 10 });
    progress.off(progressId)
  },
  download_patch11: async job => {
    const progressId = progress.on({ type: 'download-patch' }, (progress) => {
      job.progress = progress.percentage
    })
    path_patch11 = await invoke('download_patch', { patch: 11 });
    console.log('path_patch11', path_patch11)
    progress.off(progressId)
  },
  download_vcredist11: async job => {
    const progressId = progress.on({ type: 'download-vcredist' }, (progress) => {
      job.progress = progress.percentage
    })
    path_vcredist11 = await invoke('download_vcredist', { version: 11 });
    progress.off(progressId)
  },
  download_vcredist14: async job => {
    const progressId = progress.on({ type: 'download-vcredist' }, (progress) => {
      job.progress = progress.percentage
    })
    path_vcredist14 = await invoke('download_vcredist', { version: 14 });
    progress.off(progressId)
  },
  install_game: async job => {
    try {
      await invoke('install_game', { targetDir: _installDir.value, installerDir: path_unzipped });
      // wait 3 seconds for the installer to wrap up
      await new Promise(resolve => setTimeout(resolve, 3000));
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_patch10: async job => {
    try {
      await invoke('install_patch', { installerPath: path_patch10 });
      // wait 3 seconds for the installer to wrap up
      await new Promise(resolve => setTimeout(resolve, 3000));
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_patch11: async job => {
    try {
      await invoke('install_patch', { installerPath: path_patch11 });
      // wait 3 seconds for the installer to wrap up
      await new Promise(resolve => setTimeout(resolve, 3000));
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_vcredist11: async job => {
    try {
      await invoke('install_vcredist', { vcredistExe: path_vcredist11 });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_vcredist14: async job => {
    try {
      await invoke('install_vcredist', { vcredistExe: path_vcredist14 });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
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
  _installDir.value = localStorage.getItem('do-install') || _installDir.value
  console.log('install dir', _installDir.value)
  localStorage.removeItem('do-install')

  const todo: [string, Function][] = []

  const isInstalled = await invoke('get_install_path')
  let isPatched = false;
  if (!isInstalled) {
    todo.push(["Download game", jobs.download_game])
    todo.push(["Unzip game", jobs.unzip_game])
  } else {
    let version = await invoke('extract_game_version') as any;
    console.log(version)
    isPatched = version.patch == 1 && version.build == 1;
  }

  if (!isPatched) {
    todo.push(["Download Patch 10", jobs.download_patch10])
    todo.push(["Download Patch 11", jobs.download_patch11])
  }

  todo.push(["Download Visual Studio C++ Redistributable 11", jobs.download_vcredist11])
  todo.push(["Download Visual Studio C++ Redistributable 14", jobs.download_vcredist14])
  todo.push(["Install Visual Studio C++ Redistributable 11", jobs.install_vcredist11])
  todo.push(["Install Visual Studio C++ Redistributable 14", jobs.install_vcredist14])

  todo.push(["Install Game", jobs.install_game])
  if (!isInstalled) {
  }
  if (!isPatched) {
    todo.push(["Install Patch 10", jobs.install_patch10])
    todo.push(["Install Patch 11", jobs.install_patch11])
  }


  let skip = [
    // "Download game",
    // "Unzip game",
    // "Download Patch 10",
    // "Download Patch 11",
    // "Download Visual Studio C++ Redistributable 11",
    // "Download Visual Studio C++ Redistributable 14",
    // "Install Visual Studio C++ Redistributable 11",
    // "Install Visual Studio C++ Redistributable 14",
    // "Install Game",
    // "Install Patch 10",
    // "Install Patch 11",
  ]

  for (let job of todo) {
    if (_.includes(skip, job[0])) {
      continue;
    }
    await manager.runJob(job[0], job[1])
  }

  _done.value = true;
}

onMounted(async () => {
  const route = useRoute()
  if (route.params.step == 'goes') {
    _step.value = 'goes'
    goes()
  }
})
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
        <input type="text" class="form-control" id="install-location" v-model="_installDir">
      </div>
      <button @click="_step = 'goes'; goes()" class="cta">Download and install</button>
    </div>
    <div v-if="_step == 'goes'" class="card-body">
      <p style="display:block">Installing to {{ _installDir }}.</p>
      <p>Hands free once the installation process starts. Don't touch your mouse or keyboard until install is complete
      </p>
      <jobs-vue :jobs="_jobs" id="install-jobs" />
      <div v-if="_done" class="done">
        Installation complete. Next steps: <a href="https://www.massgate.org/" target="_blank"
          class="cta primary">Download
          and install the World in Conflict Multiplayer Fix
          from
          massgate.org</a>
        <router-link to="/" class="cta secondary">Back to main</router-link>
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
</style>