<script setup lang="ts">
import _ from 'lodash'

import { open } from '@tauri-apps/api/dialog';
import { appDir } from '@tauri-apps/api/path';

import { ref, reactive, onMounted, watch } from 'vue'
import EULA_game from '../../assets/eula.txt?raw'
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api';

import jobsVue from '../jobs.vue'
import wicJobs from '../../lib/wic-jobs';

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg'
import iconTriangleExclamation from '@fortawesome/fontawesome-free/svgs/solid/triangle-exclamation.svg'

const manager = wicJobs.manager
manager.clearJobs();
const progress = wicJobs.progress

const DEFAULT_INSTALL_DIR = 'C:\\Program Files (x86)\\Ubisoft\\World in Conflict'

const _installDir = ref(localStorage.getItem('install-dir') || DEFAULT_INSTALL_DIR)
const createShortcut = localStorage.getItem('create-shortcut')
const _createShortcut = ref(createShortcut !== null ? createShortcut === 'true' : true)
const _step = ref('eula')
const _done = ref(false)

const _jobs = wicJobs._jobs

let path_zipped_game = '';
let path_unzipped_game = '';
let path_patch10 = '';
let path_patch11 = '';
let path_vcredist = '';
let path_zipped_hooks = '';

// let path_zipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en.zip'
// let path_unzipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en'
// let path_patch10 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
// let path_patch11 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
// let path_vcredist11 = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_11.exe';
// let path_vcredist14 = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_14.exe';


let jobs = {
  download_game: async job => {
    const progressId = progress.on({ type: 'download-game' }, (progress) => {
      job.progress = progress.percentage
    })
    path_zipped_game = await invoke('download_game');
    progress.off(progressId)
    return true;
  },
  unzip_game: async job => {
    const progressId = progress.on({ type: 'extract-game' }, (progress) => {
      job.progress = progress.percentage
    })
    path_unzipped_game = await invoke('unzip_game', { zipPath: path_zipped_game });
    progress.off(progressId)
    return true;
  },
  download_patch10: async job => {
    const progressId = progress.on({ type: 'download-patch' }, (progress) => {
      job.progress = progress.percentage
    })
    path_patch10 = await invoke('download_patch', { patch: 10 });
    progress.off(progressId)
    return true;
  },
  download_patch11: async job => {
    const progressId = progress.on({ type: 'download-patch' }, (progress) => {
      job.progress = progress.percentage
    })
    path_patch11 = await invoke('download_patch', { patch: 11 });
    console.log('path_patch11', path_patch11)
    progress.off(progressId)
    return true;
  },
  download_vcredist: async job => {
    const progressId = progress.on({ type: 'download-vcredist' }, (progress) => {
      job.progress = progress.percentage
    })
    path_vcredist = await invoke('download_vcredist');
    progress.off(progressId)
    return true;
  },
  install_game: async job => {
    try {
      await invoke('install_game', { targetDir: _installDir.value, installerDir: path_unzipped_game });
      // wait 3 seconds for the installer to wrap up
      await new Promise(resolve => setTimeout(resolve, 3000));
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  install_patch10: async job => {
    try {
      await invoke('install_patch', { installerPath: path_patch10 });
      // wait 3 seconds for the installer to wrap up
      await new Promise(resolve => setTimeout(resolve, 3000));
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  install_patch11: async job => {
    try {
      await invoke('install_patch', { installerPath: path_patch11 });
      // wait 3 seconds for the installer to wrap up
      await new Promise(resolve => setTimeout(resolve, 3000));
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  install_vcredist: async job => {
    try {
      await invoke('install_vcredist', { vcredistExe: path_vcredist });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  set_cd_key: async job => {
    try {
      await invoke('set_cd_key');
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  download_hooks: async job => {
    const progressId = progress.on({ type: 'download-hooks' }, (progress) => {
      job.progress = progress.percentage
    })

    try {
      path_zipped_hooks = await invoke('download_hooks');
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  unzip_hooks: async job => {
    const progressId = progress.on({ type: 'extract-hooks' }, (progress) => {
      job.progress = progress.percentage
    })
    try {
      await invoke('unzip_hooks', { zipPath: path_zipped_hooks, installDir: _installDir.value });
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  create_desktop_shortcut: async job => {
    try {
      await invoke('create_desktop_shortcut')
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  },
  clean_install_directory: async job => {
    try {
      await invoke('clean_install_directory');
    } catch (error) {
      console.error("error", error);
      job.info.push(error)
      return false;
    }
    return true;
  }
}

const goes = async () => {
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('not elevated, setting install dir and elevating permissions', _installDir.value)
    localStorage.setItem('do-install', _installDir.value);
    localStorage.setItem('create-shortcut', _createShortcut.value ? 'true' : 'false');
    await invoke('elevate_permissions')
    return;
  }

  console.log('elevated, continuing installation', localStorage.getItem('do-install'))
  console.log('install dir', _installDir.value)
  localStorage.removeItem('do-install')
  localStorage.removeItem('create-shortcut')

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

  todo.push(["Download Visual Studio C++ Redistributable", jobs.download_vcredist])
  todo.push(["Install Visual Studio C++ Redistributable", jobs.install_vcredist])

  if (!isInstalled) {
    todo.push(["Install Game", jobs.install_game])
  }
  if (!isPatched) {
    todo.push(["Install Patch 10", jobs.install_patch10])
    todo.push(["Install Patch 11", jobs.install_patch11])
  }

  todo.push(["Download update", jobs.download_hooks])
  todo.push(["Install update", jobs.unzip_hooks])
  todo.push(["Set CD key", jobs.set_cd_key])
  if (_createShortcut.value) {
    todo.push(["Create desktop shortcut", jobs.create_desktop_shortcut])
  }
  todo.push(["Clean install directory", jobs.clean_install_directory])


  let skip = [
    // "Download game",
    // "Unzip game",
    // "Download Patch 10",
    // "Download Patch 11",
    // "Download Visual Studio C++ Redistributable",
    // "Install Visual Studio C++ Redistributable",
    // "Install Game",
    // "Install Patch 10",
    // "Install Patch 11",
  ]

  for (let job of todo) {
    if (_.includes(skip, job[0])) {
      manager.runJob(job[0], () => { })
      continue;
    }
    const result = await manager.runJob(job[0], job[1])
    if (!result) {
      console.error('Job failed, aborting!')
      return;
    }
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
      <div class="mb-3">
        <input type="checkbox" class="form-check-input m-1" id="create-shortcut" v-model="_createShortcut">
        <label for="install-location" class="form-label">Create desktop shortcut</label>
      </div>
      <button @click="_step = 'goes'; goes()" class="cta">Download and install</button>
    </div>
    <div v-if="_step == 'goes'" class="card-body">
      <p style="display:block">Installing to {{ _installDir }}.</p>
      <p>Hands free once the installation process starts. Don't touch your mouse or keyboard until install is complete
      </p>
      <jobs-vue :jobs="_jobs" id="install-jobs" />
    </div>
    <div id="post-install" v-if="_done">
      <div id="post-install-content">
        <div class="alert alert-success done">
          World in Conflict installed successfully
        </div>
        <div class="alert alert-danger done" v-if="_done">
          <iconTriangleExclamation class="icon" />
          You might need to reboot your computer to complete the installation!
          <iconTriangleExclamation class="icon" />
        </div>
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
  padding: 25px;
  background: rgba(0, 0, 0, 0.74);
}

#post-install-content {
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