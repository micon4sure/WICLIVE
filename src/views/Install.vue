<script setup lang="ts">
import _ from 'lodash'

import { ref, reactive, onMounted } from 'vue'
import EULA_game from '../assets/eula.txt?raw'
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api';

import jobsVue from '../components/jobs.vue'
import wicJobs from '../lib/wic-jobs';

const manager = wicJobs.manager
const progress = wicJobs.progress

// const _installDir = ref('C:\\Program Files (x86)\\Sierra Entertainment\\World in Conflict')
const _installDir = ref('C:\\002_Games\\World in Conflict')
const _step = ref('eula')
const _done = ref(false)

const _jobs = wicJobs._jobs

// let path_zipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en.zip'
// let path_unzipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en'
// let path_patch10 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
// let path_patch11 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
// let path_vcredist = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86.exe';

let path_zipped = '';
let path_unzipped = '';
let path_patch10 = '';
let path_patch11 = '';
let path_vcredist = '';

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
    progress.off(progressId)
  },
  download_vcredist: async job => {
    const progressId = progress.on({ type: 'download-vcredist' }, (progress) => {
      job.progress = progress.percentage
    })
    path_vcredist = await invoke('download_vcredist');
    progress.off(progressId)
  },
  install_game: async job => {
    try {
      await invoke('install_game', { targetDir: _installDir.value, installerDir: path_unzipped });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_patch10: async job => {
    try {
      await invoke('install_patch', { version: 10, installerDir: path_patch10 });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_patch11: async job => {
    try {
      await invoke('install_patch', { version: 11, installerDir: path_patch11 });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
  install_vcredist: async job => {
    try {
      await invoke('install_vcredist', { installerDir: path_vcredist });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
    }
  },
}

const goes = async () => {
  localStorage.setItem('force-url', '/install');
  await invoke('elevate_permissions')

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

  let isVCRedistInstalled = await invoke('check_vcredist_installed');
  if (!isVCRedistInstalled) {
    todo.push(["Download Visual Studio C++ Redistributable", jobs.download_vcredist])
  }

  if (!isInstalled) {
    todo.push(["Install Game", jobs.install_game])
  }
  if (!isPatched) {
    todo.push(["Install Patch 10", jobs.install_patch10])
    todo.push(["Install Patch 11", jobs.install_patch11])
  }


  if (!isVCRedistInstalled) {
    todo.push(["Install Visual Studio C++ Redistributable", jobs.install_vcredist])
  }

  for (let job of todo) {
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
      <div class="btn-container primary">
        <button @click="_step = 'location'" class="btn">Accept License Agreement</button>
      </div>
    </div>
    <div class="card-body" v-if="_step == 'location'">
      <div class="mb-3">
        <label for="install-location" class="form-label">Select install location</label>
        <input type="text" class="form-control" id="install-location" :value="_installDir">
      </div>
      <div class="btn-container primary">
        <button @click="_step = 'goes'; goes()" class="btn">Download and install</button>
      </div>
    </div>
    <div v-if="_step == 'goes'">
      <jobs-vue :jobs="_jobs" id="install-jobs" />
      <div v-if="_done">
        <div class="alert alert-primary" role="alert">
          Installation completed. If you haven't already, download, install and apply <a
            href="https://www.massgate.org/" target="_blank">The World in Conflict Multiplayer Fix from massgate.org</a>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
#install {
  .alert {
    border-radius: 0;
  }
}

#eula {
  white-space: pre-line;
  height: 350px;
  overflow-y: scroll;
}
</style>