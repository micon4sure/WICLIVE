<script setup lang="ts">
import _ from 'lodash'
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { computed, onMounted, reactive, ref, nextTick } from 'vue';

import jobsVue from '../jobs.vue'

const VANILLA_KEY = '3EXO-ELED-MXGY-FP5M-286R'
const SOVIET_KEY = 'LABG-U3MF-RG9G-95GB-AYTH'
const CLAN_KEY = ref(null)
const STEAM_KEY = ref(null)

const _installDir = ref('')
const _hooksVersion = ref('')

const _jobs = reactive([])

const _needsCDKey = ref(false)
const _needsVCRedist = ref(false)
const _needsPatch = ref(false)
const _needsHooks = ref(false)
const _needsHooksUpdate = ref(false)

const _display = ref(false)
const _needsAction = ref(false)

const initSetupState = async () => {
  _needsHooks.value = await invoke('needs_hooks')
  _needsHooksUpdate.value = !_needsHooks.value && await invoke('needs_hooks_update')
  console.log('needs hooks', _needsHooks.value, _needsHooksUpdate.value)

  const key = _cdKey.value = await invoke('get_cd_key')
  _needsCDKey.value = !key || key == "invalid"
  _needsVCRedist.value = await invoke('needs_vc_redist')

  let version = await invoke('extract_game_version') as any;
  _needsPatch.value = version.patch != 1 || version.build != 1;

  _needsAction.value = _needsHooks.value || _needsHooksUpdate.value || _needsCDKey.value || _needsVCRedist.value || _needsPatch.value

  if (localStorage.getItem('do-install-hooks') == "true") {
    _display.value = true
    _needsAction.value = true
    localStorage.removeItem('do-install-hooks')
    installHooks()
  }

  _installDir.value = await invoke('get_install_path')
  _hooksVersion.value = await invoke('get_hooks_version')
}

onMounted(async () => {
  await initSetupState()
  _display.value = _needsAction.value

  CLAN_KEY.value = await invoke('get_secret', { secret: 'SECRET_CLAN_KEY' })
  STEAM_KEY.value = await invoke('get_secret', { secret: 'SECRET_STEAM_KEY' })
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

const installHooks = async () => {
  console.log('running install hooks')

  // DOWNLOAD HOOKS
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('elevating permissions to install hooks')
    localStorage.setItem('do-install-hooks', "true");
    await invoke('elevate_permissions')
    return;
  }

  let hooksZipPath;
  const jobDownload = reactive({
    title: 'Download Update',
    status: 'pending',
    info: [],
    progress: 0
  })
  try {
    _jobs.push(jobDownload)

    listen('download-progress', (progress: any) => {
      const payload = JSON.parse(progress.payload)
      if (payload.type != 'download-hooks') return
      jobDownload.progress = payload.percentage
    })
    hooksZipPath = await invoke('download_hooks')
    jobDownload.status = 'success'
  } catch (e) {
    jobDownload.status = 'error'
    jobDownload.info.push(e)
    return
  }

  // INSTALL HOOKS
  const jobInstall = reactive({
    title: 'Unzip/install update',
    status: 'pending',
    info: [],
    progress: 0
  })
  try {
    listen('extract-hooks', (progress: any) => {
      const payload = JSON.parse(progress.payload)
      if (payload.type != 'download-hooks') return
      jobDownload.progress = payload.percentage
    })

    _jobs.push(jobInstall)
    await invoke('unzip_hooks', { zipPath: hooksZipPath });
    jobInstall.status = 'success'
  } catch (e) {
    jobInstall.status = 'error'
    jobInstall.info.push("Failed to unzip the update. Is World in Conflict running?")
    jobInstall.info.push(e)
  }

  initSetupState()
}
</script>

<template>
  <!-- {{ {
    _needsHooks,
    _needsHooksUpdate,
    _needsCDKey,
    _needsVCRedist,
    _needsPatch,
    _needsAction,
    _display,
    _jobs,
    _cdKey,
    _errorSetCDKey,
    _isInConfirmModeVanilla,
    _isInConfirmModeSoviet,
    VANILLA_KEY,
    SOVIET_KEY,
    CLAN_KEY,
    STEAM_KEY
  } }} -->
  <div id="setup" v-if="_display" class="mb-5">
    <h2>Setup</h2>
    <div id="setup-flex">
      <div id="setup-container" v-if="_needsAction">

        <div class="card mb-3" v-if="_needsHooks">
          <div class="card-header">Multiplayer fix</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is not correctly configured to play multiplayer. You need to install
              the
              multiplayer update.
            </p>
            <p>
              Your install directory is: {{ _installDir }}
            </p>
            <p>
              <small>If you have the massgate.org multiplayer fix installed, this action will overwrite it.</small>
            </p>
            <button @click="installHooks" class="cta">Install update</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsHooksUpdate">
          <div class="card-header">Multiplayer update</div>
          <div class="card-body">
            <p>
              Your World in Conflict multiplayer update is outdated. You need to update it to the latest version.
            </p>
            <button @click="installHooks" class="cta">Update</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsCDKey">
          <div class="card-header">CD Key [current: {{ _cdKey }}]</div>
          <div class="card-body">
            <p>
              Your CD key is not valid for play on the MASSGATE service. You need to set a valid CD key to play
              online.
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
                <div>Soviet Assault / Complete Edition<br />{{ SOVIET_KEY }}</div>
                <button class="cta small secondary" @click="_isInConfirmModeSoviet = true"
                  v-if="_isInConfirmModeSoviet == false">Write to registry</button>
                <button class="cta small primary" @click="setCDKey(SOVIET_KEY)" v-else>Confirm Write to
                  registry</button>
              </div>
              <div class="set-cdkey-option" v-if="CLAN_KEY">
                <div>Clan Edition<br />{{ CLAN_KEY }}</div>
                <button class="cta small primary" @click="setCDKey(CLAN_KEY)">Confirm Write to
                  registry</button>
              </div>
              <div class="set-cdkey-option" v-if="STEAM_KEY">
                <div>Steam Edition<br />{{ STEAM_KEY }}</div>
                <button class="cta small primary" @click="setCDKey(STEAM_KEY)">Confirm Write to
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
              Your World in Conflict installation is missing the Visual Studio C++ Redistributable. This is required
              to
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
  <div v-else>
    <div id="setup-container">
      <div class="card mb-3">
        <div class="card-header">Setup state</div>
        <div class="card-body">
          <p>
            WIC LIVE is using this install directory: {{ _installDir }}
          </p>
          <p>
            The version of your update is {{ _hooksVersion }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
#setup-flex {
  display: flex;

  .card {
    flex: 1;
  }
}

#setup .jobs {
  min-width: 35%;
}
</style>