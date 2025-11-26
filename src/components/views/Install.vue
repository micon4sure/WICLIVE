<script setup lang="ts">
import { open } from '@tauri-apps/api/dialog';
import _ from 'lodash'

import { ref, reactive, onMounted, watch } from 'vue'
// import EULA_game from '../../assets/eula.txt?raw'
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api';

import jobsVue from '../jobs.vue'

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg'
import iconTriangleExclamation from '@fortawesome/fontawesome-free/svgs/solid/triangle-exclamation.svg'

import useInstallJobs from '../../lib/install-jobs'
import { Job } from '../../lib/wic-job';
const installJobs = useInstallJobs();
const _installDir = installJobs._installDir;

// let path_zipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en.zip'
// let path_unzipped = 'C:\\Users\\micon\\AppData\\Local\\Temp\\world_in_conflict_retail_1.000_en'
// let path_patch10 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
// let path_patch11 = 'C:\\Users\\micon\\AppData\\Local\\Temp';
// let path_vcredist11 = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_11.exe';
// let path_vcredist14 = 'C:\\Users\\micon\\AppData\\Local\\Temp\\vcredist_x86_14.exe';


const _step = ref('goes')
const _done = ref(false)
const _todo: Job[] = reactive<Job[]>([])
const createShortcut = localStorage.getItem('create-shortcut')
const _createShortcut = ref(createShortcut !== null ? createShortcut === 'true' : true)

const _isElevated = ref(false);

const goes = async () => {
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('not elevated, setting install dir and elevating permissions', _installDir.value)
    localStorage.setItem('do-install', _installDir.value);
    localStorage.setItem('create-shortcut', _createShortcut.value ? 'true' : 'false');
    await invoke('elevate_permissions')
    return;
  }
  _isElevated.value = true;

  console.log('elevated, continuing installation', localStorage.getItem('do-install'))
  console.log('install dir', _installDir.value)
  localStorage.removeItem('do-install')
  localStorage.removeItem('create-shortcut')

  // game installation disabled
  // const isInstalled = await invoke('get_install_path')
  // if (!isInstalled) {
  //   todo.push(download_game)
  //   todo.push(unzip_game)
  // } else {
  // }
  // if (!isInstalled) {
  //   todo.push(install_game)
  // }

  _todo.push(installJobs.clean_temp_directory_pre)

  let version = await invoke('extract_game_version') as any;
  const isPatched = version.patch == 1 && version.build == 1;

  if (!isPatched) {
    if (version.patch == 0)
      _todo.push(installJobs.download_patch10)
    _todo.push(installJobs.download_patch11)
  }

  _todo.push(installJobs.download_vcredist)
  _todo.push(installJobs.install_vcredist)

  if (!isPatched) {
    if (version.patch == 0)
      _todo.push(installJobs.install_patch10)
    _todo.push(installJobs.install_patch11)
  }

  _todo.push(installJobs.download_hooks)
  _todo.push(installJobs.unzip_hooks)
  _todo.push(installJobs.set_cd_key)
  _todo.push(installJobs.set_laa)
  if (_createShortcut.value) {
    _todo.push(installJobs.create_desktop_shortcut)
  }

  _todo.push(installJobs.clean_temp_directory_post)

  let allSuccessful = true;
  for (let job of _todo) {
    try {
      await job.run();
    }
    catch (e) {
      console.error('Installation failed at job', job.data.title, e);
      allSuccessful = false;
      break;
    }
  }
  _done.value = allSuccessful;
}


onMounted(async () => {
  const route = useRoute()
  if (route.params.step == 'goes') {
    _step.value = 'goes'
    goes()
  }
})

// watch(_installDir, (val) => {
//   console.log('SETTING INSTALL DIR', val)
//   localStorage.setItem('install-dir', val)
// })
// const selectInstallDir = async () => {
//   const selected = await open({
//     multiple: false,
//     directory: true,
//     defaultPath: _installDir.value,
//   });
//   if (selected === null) {
//     // user cancelled the selection
//   } else {
//     _installDir.value = selected as string
//   }
// }
</script>

<template>
  <div class="card" id="install">
    <div class="card-header">
      Install World in Conflict
    </div>
    <!-- <div class="card-body" v-if="_step == 'eula'">
      <div id="eula">
        {{ EULA_game }}
      </div>
      <button @click="_step = 'location'" class="btn cta">Accept License Agreement</button>
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
      <button @click="_step = 'goes'; goes()" class="btn cta">Download and install</button>
    </div> -->
    <div v-if="!_isElevated" class="card-body">
      <h5>Requesting elevated permissions...</h5>
      <p>WIC LIVE needs administrator rights to:</p>
      <ul>
        <li>Install game patches to the Program Files directory</li>
        <li>Install Visual Studio C++ Redistributable</li>
        <li>Install the multiplayer fix</li>
        <li>Modify wic.exe for Large Address Aware support</li>
      </ul>
      <p><strong>Please accept the Windows <em>User Account Control</em> prompt to continue.</strong></p>
    </div>
    <div v-else-if="_step == 'goes'" class="card-body">
      <p style="display:block">Installing to {{ _installDir }}.</p>
      <p>Hands free once the installation process starts. Don't touch your mouse or keyboard until install is complete
      </p>
      <jobs-vue :jobs="_todo" id="install-jobs" />
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
