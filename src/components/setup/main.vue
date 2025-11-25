<script setup lang="ts">
import _ from 'lodash'
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { computed, onMounted, reactive, ref, nextTick } from 'vue';

import jobsVue from '../jobs.vue'
import axios from 'axios';

const VANILLA_KEY = '3EXO-ELED-MXGY-FP5M-286R'
const SOVIET_KEY = 'LABG-U3MF-RG9G-95GB-AYTH'
const CLAN_KEY = ref(null)
const STEAM_KEY = ref(null)

const _isInitialized = ref(false)
const _initializeSuccess = ref(true)

const _installDir = ref('')
const _hooksVersion = ref('')

const _jobsFixes = reactive([])
const _jobsInit = reactive([])

const _needsCDKey = ref(false)
const _needsVCRedist = ref(false)
const _needsPatch = ref(false)
const _needsHooks = ref(false)
const _needsHooksUpdate = ref(false)
const _needsLAA = ref(false)

const _needsAction = ref(false)
const _display = ref(false)

const _isSoviet = ref(false)

const checkInit = () => {
  let initialized = true;
  for (const job of _jobsInit) {
    if (job.status == 'pending') {
      console.log('STILL PENDING')
      initialized = false;
    }
  }

  if (initialized) {
    _isInitialized.value = true
    onInitialized();
  }
}

const addInitJob = async (title, action) => {
  await addJob('init', title, action)
}
const addFixJob = async (title, action) => {
  await addJob('fix', title, action)
}

const addJob = async (type: 'init' | 'fix', title, action) => {

  return new Promise<void>(async (resolve, reject) => {
    const jobs = type == 'init' ? _jobsInit : _jobsFixes
    const job = reactive({
      title,
      status: 'pending',
      info: [],
      progress: 0
    })

    const setListener = progressID => {
      listen('download-progress', (progress: any) => {
        const payload = JSON.parse(progress.payload)
        if (payload.type != 'download-vcredist') return
        job.progress = payload.percentage
      })
    }

    jobs.push(job)
    try {
      await action(setListener);
      job.status = 'success'
    } catch (e) {
      job.info.push(e)
      job.status = "error"
      if (type == 'init')
        _initializeSuccess.value = false
    } finally {
      if (type == 'init')
        checkInit()
      else
        initSetupState()
    }
    resolve();
  })
}

const initSetupState = async () => {
  while (_jobsInit.length > 0)
    _jobsInit.pop()

  await nextTick()

  addInitJob("Check need game patches", async () => {
    let version = await invoke('extract_game_version') as any;
    _needsPatch.value = version.patch != 1 || version.build != 1;
  })

  addInitJob("Check CD key", async () => {
    const key = _cdKey.value = await invoke('get_cd_key')
    _needsCDKey.value = !key || key == "invalid"
    _needsVCRedist.value = await invoke('needs_vc_redist')
  })

  addInitJob("Check LAA flag", async () => {
    const laaFlag = await invoke('get_laa_flag') as boolean;
    _needsLAA.value = !laaFlag;
  })

  addInitJob('Check need multiplayer update', async () => {
    _needsHooks.value = await invoke('needs_hooks')
    _hooksVersion.value = (await axios.get('https://www.wicgate.com/wic_cl_hook-version.txt')).data
    _needsHooksUpdate.value = !_needsHooks.value && await invoke('needs_hooks_update', { version: _hooksVersion.value })
  })
}

const onInitialized = async () => {
  _needsAction.value = _needsHooks.value || _needsHooksUpdate.value || _needsCDKey.value || _needsVCRedist.value || _needsPatch.value || _needsLAA.value
  if (_needsAction.value)
    _display.value = true;

  if (localStorage.getItem('do-install-hooks') == "true") {
    _needsAction.value = true
    localStorage.removeItem('do-install-hooks')
    installHooks()
  }

  _installDir.value = await invoke('get_install_path')
  _isSoviet.value = await invoke('is_soviet_assault')
}


onMounted(async () => {
  await initSetupState()

  CLAN_KEY.value = await invoke('get_secret', { secret: 'SECRET_CLAN_KEY' })
  STEAM_KEY.value = await invoke('get_secret', { secret: 'SECRET_STEAM_KEY' })
})

// SET CD KEY
const _cdKey = ref('')
const _errorSetCDKey = ref(null)
const setCDKey = async (key: string) => {
  addFixJob('Set CD Key', async () => {
    await invoke('set_cd_key', { key })
    const confirmKey = await invoke('get_cd_key')
    if (confirmKey !== key)
      throw new Error('CD Key not set correctly')
    _cdKey.value = key
  })
}

// INSTALL VC REDIST
const installVCRedist = async () => {
  let redistPath;

  await addFixJob('Install Visual Studio C++ Redistributable', async (setListener) => {
    setListener('download-vcredist')
    redistPath = await invoke('download_vcredist')
  });

  addFixJob('Installing Visual Studio C++ Redistributable', async () => {
    await invoke('install_vcredist', { vcredistExe: redistPath });
    _needsVCRedist.value = await invoke('needs_vc_redist')
  });
}

// INSTALL HOOKS
const installHooks = async () => {
  console.log('running install hooks')

  // elevate
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('elevating permissions to install hooks')
    localStorage.setItem('do-install-hooks', "true");
    await invoke('elevate_permissions')
    return;
  }

  // download
  let hooksZipPath;
  await addFixJob('Download multiplayer update', async (setListener) => {
    setListener('download-hooks')
    const latestHooksVersion = (await axios.get('https://www.wicgate.com/wic_cl_hook-version.txt')).data
    hooksZipPath = await invoke('download_hooks', { version: latestHooksVersion })
  });

  // install
  await addFixJob('Unzip/install multiplayer update', async () => {
    await invoke('unzip_hooks', { zipPath: hooksZipPath });
  });

  initSetupState()
}

// FIX LAA
const fixLAA = async () => {
  addFixJob('Set Large Address Aware flag', async () => {
    await invoke('set_laa_flag')
    const confirmLAA = await invoke('get_laa_flag')
    if (!confirmLAA)
      throw new Error('LAA flag not set correctly')
  });

  initSetupState();
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
  <div id="setup" class="mb-5">
    <h2>Setup</h2>
    <div>
      <div class="setup-flex">
        <div class="setup-controls">
          <div class="card mb-3">
            <div class="card-header">Installation state</div>
            <div class="card-body" v-if="!_initializeSuccess">
              <p class="big-error">
                There were errors during initialization. Please restart WIC LIVE to try again.
              </p>
            </div>
            <div class="card-body" v-else-if="_installDir && _hooksVersion">
              <p>
                WIC LIVE is using this install directory: <strong>{{ _installDir }}</strong>
              </p>
              <p>The edition is: <strong v-if="_isSoviet">Soviet Assault</strong><strong v-else>Vanilla</strong></p>
              <p>
                The version of your update is <strong>{{ _hooksVersion }}</strong>
              </p>
              <p v-if="!_needsAction">
                <strong>You're all set for online multiplayer</strong>
              </p>
            </div>
            <div class="card-body" v-else-if="!_isInitialized">
              <div class="spinner-border" role="status">
                <span class="sr-only">&nbsp;</span>
              </div>
              <strong>Initializing...</strong>
            </div>
            <div class="card-body" v-else>
              World in Conflict is not installed.
            </div>
          </div>
        </div>
        <jobs-vue :jobs="_jobsInit" />
      </div>
    </div>
    <div class="setup-flex" v-if="_initializeSuccess && _display">
      <div class="setup-controls">
        <div class="card mb-3" v-if="_needsHooks">
          <div class="card-header">Enable online multiplayer</div>
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
            <button @click="installHooks" class="btn cta">Install update</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsLAA">
          <div class="card-header">Game patch required</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is not <em>Large Adress Aware</em>. This can cause problems when
              starting or running the game.
            </p>
            <button @click="fixLAA" class="btn cta">Fix problem</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsHooksUpdate">
          <div class="card-header">Multiplayer update</div>
          <div class="card-body">
            <p>
              Your World in Conflict multiplayer update is outdated. You need to update it to the latest version.
            </p>
            <button @click="installHooks" class="btn cta">Update</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsCDKey">
          <div class="card-header">CD Key <span v-if="_cdKey">[current: {{ _cdKey }}]</span></div>
          <div class="card-body">
            <p>
              Your CD key is not valid for play on WICGATE service. You need to set a valid CD key to play
              online.
            </p>
            <p v-if="!_isSoviet">Since you have the Vanilla Edition installed, we suggest the following key: <strong>{{
              VANILLA_KEY }}</strong></p>
            <p v-else>Since you have the Soviet Assault / Complete Edition installed, we suggest the following key:
              <strong>{{ SOVIET_KEY }}</strong>
            </p>
            <p>
              If you run into problems with the key, you can change it later the `Config` section below.
            </p>
            <button class="btn cta" @click="setCDKey(_isSoviet ? SOVIET_KEY : VANILLA_KEY)">Write CD Key to
              registry</button>
          </div>
          <div class="card-body" id="set-cdkey" v-if="CLAN_KEY || STEAM_KEY">
            <div id="set-cdkey-options">
              <div class="set-cdkey-option" v-if="CLAN_KEY">
                <div>Clan Edition<br />{{ CLAN_KEY }}</div>
                <button class="btn cta small primary" @click="setCDKey(CLAN_KEY)">Write to
                  registry</button>
              </div>
              <div class="set-cdkey-option" v-if="STEAM_KEY">
                <div>Steam Edition<br />{{ STEAM_KEY }}</div>
                <button class="btn cta small primary" @click="setCDKey(STEAM_KEY)">Write to
                  registry</button>
              </div>
              <div class="bg-danger p-3" v-if="_errorSetCDKey">{{ _errorSetCDKey }}</div>
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
              <button class="btn cta" @click="installVCRedist">Install VC Redist</button>
            </div>
          </div>
        </div>
        <div class="card" v-if="!_needsAction">
          <div class="card-header">Setup complete</div>
          <div class="card-body">
            <p>All manual fixes applied.</p>
          </div>
        </div>
      </div>
      <jobs-vue :jobs="_jobsFixes" />
    </div>
  </div>

</template>

<style lang="scss" scoped>
.setup-flex {
  display: flex;

  .setup-controls {
    flex: 1;
  }
}

.spinner-border {
  width: 30px;
  height: 30px;
  margin: 0 10px;
}

.big-error {
  background: red;
  border-radius: 5px;
  padding: 10px;
  font-weight: bold;
  font-size: 1.2em;
}
</style>