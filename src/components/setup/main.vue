<script setup lang="ts">
import _ from 'lodash'
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { computed, onMounted, reactive, ref, nextTick } from 'vue';

import jobsVue from '../jobs.vue'

const VANILLA_KEY = '3EXO-ELED-MXGY-FP5M-286R'
const SOVIET_KEY = 'LABG-U3MF-RG9G-95GB-AYTH'


const _jobs = reactive([])


const _hasHookFiles = ref(false)
const _needsMulticoreFix = ref(false)
const _needsHostsEntries = ref(false)
const _needsCDKey = ref(false)
const _needsVCRedist = ref(false)
const _needsPatch = ref(false)
const initSetupState = async () => {
  _hasHookFiles.value = await invoke('has_hook_files')
  _needsHostsEntries.value = await invoke('needs_hosts_entries')
  _needsMulticoreFix.value = await invoke('needs_multicore_fix')
  const key = _cdKey.value = await invoke('get_cd_key')
  _needsCDKey.value = key !== VANILLA_KEY && key !== SOVIET_KEY
  _needsVCRedist.value = await invoke('needs_vc_redist')

  let version = await invoke('extract_game_version') as any;
  _needsPatch.value = version.patch != 1 || version.build != 1;

}


const applyMulticore = async () => {
  const job = reactive({
    title: 'Apply multicore fix',
    status: 'pending',
    info: [],
    progress: 0
  })
  _jobs.push(job)

  try {
    await invoke('apply_multicore_fix')
    job.status = 'success'
  } catch (e) {
    job.status = 'error'
    job.info.push(e)
  }

  initSetupState()
}

const removeHookFiles = async () => {
  const job = reactive({
    title: 'Remove obsolete files',
    status: 'pending',
    info: [],
    progress: 0
  })
  _jobs.push(job)

  try {
    await invoke('remove_hook_files')
    job.status = 'success'
  } catch (e) {
    job.status = 'error'
    job.info.push(e)
  }

  initSetupState()
}

const addHostsEntries = async () => {
  const isElevated = await invoke('is_elevated')
  if (!isElevated) {
    localStorage.setItem('do-add-hosts-entries', 'true')
    invoke('elevate_permissions')
    return
  }

  const job = reactive({
    title: 'Add hosts entries',
    status: 'pending',
    info: [],
    progress: 0
  })
  _jobs.push(job)

  try {
    await invoke('add_hosts_entries');
    job.status = 'success'
  }
  catch (e) {
    job.status = 'error'
    job.info.push(e)
  }

  _needsHostsEntries.value = await invoke('needs_hosts_entries')
}
const _display = ref(false)
const _needsAction = computed(() => {
  return _needsHostsEntries.value || _needsMulticoreFix.value || _hasHookFiles.value || _needsCDKey.value || _needsVCRedist.value
});
onMounted(async () => {
  await initSetupState()
  _display.value = _needsAction.value

  if (localStorage.getItem('do-add-hosts-entries')) {
    localStorage.removeItem('do-add-hosts-entries')
    addHostsEntries()
  }
})

const _cdKey = ref('')
const _errorSetCDKey = ref(null)

const setCDKey = async (key: string) => {
  await invoke('set_cd_key', { key })

  try {
    const confirmKey = await invoke('get_cd_key')
    if (confirmKey !== key)
      throw new Error('CD Key not set correctly')
    _cdKey.value = key

    _isInConfirmModeVanilla.value = false
    _isInConfirmModeSoviet.value = false
  } catch (error) {
    _errorSetCDKey.value = error
  }

  initSetupState()
}

const _isInConfirmModeVanilla = ref(false)
const _isInConfirmModeSoviet = ref(false)

const installVCRedist = async () => {
  let redistPath;
  const jobDownload = reactive({
    title: 'Download Visual Studio C++ Redistributable',
    status: 'pending',
    info: [],
    progress: 0
  })
  try {
    _jobs.push(jobDownload)


    listen('download-progress', (progress: any) => {
      const payload = JSON.parse(progress.payload)
      if (payload.type != 'download-vcredist') return
      jobDownload.progress = payload.percentage
    })
    redistPath = await invoke('download_vcredist')
    jobDownload.status = 'success'
  } catch (e) {
    jobDownload.status = 'error'
    jobDownload.info.push(e)
    return
  }

  const jobInstall = reactive({
    title: 'Install Visual Studio C++ Redistributable',
    status: 'pending',
    info: [],
    progress: 0
  })
  try {
    _jobs.push(jobInstall)
    await invoke('install_vcredist', { vcredistExe: redistPath });
  } catch (e) {
    jobInstall.status = 'error'
    jobInstall.info.push(e)
  }

  _needsVCRedist.value = await invoke('needs_vc_redist')
}
</script>

<template>
  <div id="setup" v-if="_display" class="mb-5">
    <h2>Setup</h2>
    <div id="setup-flex">
      <div id="setup-container" v-if="_needsAction">

        <div class="card mb-3" v-if="!_hasHookFiles && _needsHostsEntries">
          <div class="card-header">Massgate fix</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is not yet set up for online play. We need to set some entries in the
              hosts file to make multiplayer work. This needs administrator rights. You will be prompted for permission
              elevation.
            </p>
            <button @click="addHostsEntries" class="cta">Add hosts entries</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_hasHookFiles">
          <div class="card-header">Massgate fix</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is set up for the obsolete MASSGATE service. We need to remove
              obsolete files to make multiplayer work.
            </p>
            <button @click="removeHookFiles" class="cta">Remove files</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsMulticoreFix">
          <div class="card-header">Multicore fix</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation needs a fix to run on your CPU because it has more than cores/threads
              than
              the game can handle.
            </p>
            <button @click="applyMulticore" class="cta">Install fix</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsCDKey">
          <div class="card-header">CD Key [current: {{ _cdKey }}]</div>
          <div class="card-body">
            <p>
              Your CD key is not valid for play on the MASSGATE service. You need to set a valid CD key to play online.
            </p>
            <p>
              If you have the `Soviet Assault` version from Steam or the `Complete Edition` from GOG, use the `Soviet
              Assault` key.<br />
              Otherwise, just use the `Vanilla` key.
            </p>
            <p>
              If you run into problems with the key, you can change it later inn the `Config` section below.
            </p>
          </div>
          <div class="card-body" id="set-cdkey">
            <div id="set-cdkey-options">
              <div class="set-cdkey-option">
                <div>Vanilla Edition<br />{{ VANILLA_KEY }}</div>
                <button class="cta small secondary" @click="_isInConfirmModeVanilla = true"
                  v-if="_isInConfirmModeVanilla == false">Write to registry</button>
                <button class="cta small primary" @click="setCDKey(VANILLA_KEY)" v-else>Confirm Write to
                  registry</button>
              </div>
              <div class="set-cdkey-option">
                <div>Soviet Assault<br />{{ SOVIET_KEY }}</div>
                <button class="cta small secondary" @click="_isInConfirmModeSoviet = true"
                  v-if="_isInConfirmModeSoviet == false">Write to registry</button>
                <button class="cta small primary" @click="setCDKey(SOVIET_KEY)" v-else>Confirm Write to
                  registry</button>
              </div>
              <div class="bg-danger p-3" v-if="_errorSetCDKey">{{ _errorSetCDKey }}</div>
            </div>
          </div>
        </div>

        <div class="card" v-if="_needsVCRedist">
          <div class="card-header">Visual Studio C++ Redistributable is missing</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is missing the Visual Studio C++ Redistributable. This is required to
              run the game.
            </p>
            <button class="cta" @click="installVCRedist">Install VC Redist</button>
          </div>
        </div>
      </div>
      <div class="card" v-if="!_needsAction">
        <div class="card-header">Setup complete</div>
        <div class="card-body">
          <p>Your World in Conflict installation is set up correctly.</p>
        </div>
      </div>
      <jobs-vue :jobs="_jobs" />

    </div>
  </div>
</template>

<style lang="scss">
#setup-flex {
  display: flex;
}

#setup .jobs {
  min-width: 35%;
}
</style>