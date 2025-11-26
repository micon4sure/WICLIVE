<script setup lang="ts">
import _ from 'lodash'
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { computed, onMounted, reactive, ref, nextTick } from 'vue';


import jobsVue from '../jobs.vue'
import axios from 'axios';
import { Job, Job_Data } from '../../lib/wic-job';
import useInstallJobs from '../../lib/install-jobs'

const installJobs = useInstallJobs();

const VANILLA_KEY = '3EXO-ELED-MXGY-FP5M-286R'
const SOVIET_KEY = 'LABG-U3MF-RG9G-95GB-AYTH'
const CLAN_KEY = ref(null)
const STEAM_KEY = ref(null)

const _isInitialized = ref(false)
const _initializeSuccess = ref(true)

const _installDir = ref('')
const _gameVersion = ref(null)
const _hooksVersion = ref('')

const _jobsInit: Job[] = reactive([])
const _jobsFixes: Job[] = reactive([])

const _needsCDKey = ref(false)
const _needsVCRedist = ref(false)
const _needsPatch = ref(false)
const _needsHooks = ref(false)
const _needsHooksUpdate = ref(false)
const _needsLAA = ref(false)

const _needsAction = ref(false)
const _display = ref(false)

const _isSoviet = ref(false)

// Track running state of fix actions
const _isFixingPatches = ref(false)
const _isFixingHooks = ref(false)
const _isFixingLAA = ref(false)
const _isFixingVCRedist = ref(false)
const _isFixingCDKey = ref(false)

const checkInit = () => {
  let initialized = true;
  for (const job of _jobsInit) {
    if (job.data.status == 'queued' || job.data.status == 'running') {
      initialized = false;
    }
  }

  if (initialized) {
    _isInitialized.value = true
    onInitialized();
  }
}

const runInitJob = async (title, action) => {
  await createJob('init', title, action).run()
}
const runFixJob = async (title, action, progressFilter: string | null = null) => {
  await createJob('fix', title, action, progressFilter).run()
}
const createFixJob = (title, action, progressFilter: string | null = null) => {
  return createJob('fix', title, action, progressFilter)
}

const createJob = (type: 'init' | 'fix', title, action, progressFilter: string | null = null) => {
  const data: Job_Data = reactive({
    title,
    status: 'queued',
    info: [],
    progress: 0
  })
  const jobs = type == 'init' ? _jobsInit : _jobsFixes

  const run = async () => {
    let unlisten: (() => void) | null = null


    data.status = 'running'

    if (progressFilter) {
      unlisten = await listen('download-progress', (event: any) => {
        const payload = JSON.parse(event.payload)
        if (payload.type !== progressFilter) return
        data.progress = payload.percentage
      })
    }

    const addInfo = (text: string, highlight: boolean = false) => {
      data.info.push({ text, highlight })
    }

    try {
      await action({ addInfo });
      data.status = 'success'
    } catch (e) {
      data.info.push({ text: String(e), highlight: true })
      data.status = "error"
      if (type == 'init')
        _initializeSuccess.value = false
    } finally {
      if (unlisten) unlisten()
      if (type == 'init')
        checkInit()
      else
        initSetupState()
    }
  }
  const job = { data, run }
  jobs.push(job)
  return job
}

const initSetupState = async () => {
  while (_jobsInit.length > 0)
    _jobsInit.pop()

  await nextTick()

  runInitJob("Check need game patches", async ({ addInfo }) => {
    let version = await invoke('extract_game_version') as any;
    _gameVersion.value = version.major + '.' + version.minor + '.' + version.patch + '.' + version.build;
    addInfo(`Version: ${_gameVersion.value}`)
    _needsPatch.value = version.patch != 1 || version.build != 1;
    if (_needsPatch.value)
      addInfo('Game is not patched to latest version', true)
  })
  runInitJob("Check LAA flag", async ({ addInfo }) => {
    const laaFlag = await invoke('get_laa_flag') as boolean;
    _needsLAA.value = !laaFlag;
    addInfo(laaFlag ? 'Enabled' : 'Not set', _needsLAA.value)
  })


  runInitJob('Check need Visual Studio C++ Redistributable', async ({ addInfo }) => {
    _needsVCRedist.value = await invoke('needs_vc_redist')
    addInfo(_needsVCRedist.value ? 'Not installed' : 'Installed', _needsVCRedist.value)
  })

  runInitJob("Check CD key", async ({ addInfo }) => {
    const key = _cdKey.value = await invoke('get_cd_key')
    _needsCDKey.value = !key || key == "invalid"
    addInfo(_needsCDKey.value ? 'Missing or invalid' : key, _needsCDKey.value)
    _needsVCRedist.value = await invoke('needs_vc_redist')
  })

  runInitJob('Check need multiplayer fix', async ({ addInfo }) => {
    _needsHooks.value = await invoke('needs_hooks')
    _hooksVersion.value = (await axios.get('https://www.wicgate.com/wic_cl_hook-version.txt')).data
    _needsHooksUpdate.value = !_needsHooks.value && await invoke('needs_hooks_update', { version: _hooksVersion.value })
    if (_needsHooks.value) addInfo('Not installed', true)
    else if (_needsHooksUpdate.value) addInfo(`Update available: ${_hooksVersion.value}`, true)
    else addInfo(`Up to date: ${_hooksVersion.value}`)
  })
}

const onInitialized = async () => {
  _needsAction.value = _needsHooks.value || _needsHooksUpdate.value || _needsCDKey.value || _needsVCRedist.value || _needsLAA.value || _needsPatch.value;
  if (_needsAction.value)
    _display.value = true;

  // Check for pending elevated actions
  if (localStorage.getItem('do-install-hooks') == "true") {
    _needsAction.value = true
    localStorage.removeItem('do-install-hooks')
    installHooks()
  }

  if (localStorage.getItem('do-fix-laa') == "true") {
    _needsAction.value = true
    localStorage.removeItem('do-fix-laa')
    fixLAA()
  }

  if (localStorage.getItem('do-install-vcredist') == "true") {
    _needsAction.value = true
    localStorage.removeItem('do-install-vcredist')
    installVCRedist()
  }

  _installDir.value = await invoke('get_install_path')
  _isSoviet.value = await invoke('is_soviet_assault')
}

onMounted(async () => {
  await initSetupState()

  CLAN_KEY.value = await invoke('get_secret', { secret: 'SECRET_CLAN_KEY' })
  STEAM_KEY.value = await invoke('get_secret', { secret: 'SECRET_STEAM_KEY' })
})

const installPatches = async () => {
  _isFixingPatches.value = true
  console.log('running install patches')

  // Elevate permissions if needed (writes to Program Files)
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('elevating permissions to install patches')
    localStorage.setItem('do-install-patches', "true");
    await invoke('elevate_permissions')
    return;
  }

  const todo: Job[] = []

  let version = await invoke('extract_game_version') as any;
  console.log('GAME VERSION:', version)
  if (version.patch == 0)
    todo.push(createFixJob('Download patch 1.0.1.0', async () => {
      await installJobs.download_patch10.run();
    }, 'download-patch'));

  todo.push(createFixJob('Download patch 1.0.1.1', async () => {
    await installJobs.download_patch11.run();
  }, 'download-patch'));

  if (version.patch == 0) {
    todo.push(createFixJob('Install patch 1.0.1.0', async () => {
      await installJobs.install_patch10.run();
    }));
  }

  todo.push(createFixJob('Install patch 1.0.1.1', async () => {
    await installJobs.install_patch11.run();
  }));

  todo.push(createFixJob('Clean temp directory', async () => {
    await invoke('clean_temp_directory');
  }));

  for (const job of todo) {
    if (!job) continue;
    await job.run();
  }
};

// SET CD KEY
const _cdKey = ref('')
const _errorSetCDKey = ref(null)
const setCDKey = async (key: string) => {
  _isFixingCDKey.value = true
  await runFixJob('Set CD Key', async () => {
    await invoke('set_cd_key', { key })
    const confirmKey = await invoke('get_cd_key')
    if (confirmKey !== key)
      throw new Error('CD Key not set correctly')
    _cdKey.value = key
  })
  _isFixingCDKey.value = false
}

// INSTALL VC REDIST
const installVCRedist = async () => {
  _isFixingVCRedist.value = true
  // Elevate permissions if needed (VC Redist install requires admin)
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('elevating permissions to install VC Redist')
    localStorage.setItem('do-install-vcredist', "true");
    await invoke('elevate_permissions')
    return;
  }

  await runFixJob('Download Visual Studio C++ Redistributable', async () => {
    await installJobs.download_vcredist.run();
  }, 'download-vcredist');

  await runFixJob('Installing Visual Studio C++ Redistributable', async () => {
    await installJobs.install_vcredist.run();
  });

  await runFixJob('Clean temp directory', async () => {
    await invoke('clean_temp_directory');
  });
  _isFixingVCRedist.value = false
}

// INSTALL HOOKS
const installHooks = async () => {
  _isFixingHooks.value = true
  console.log('running install hooks')

  // Elevate permissions if needed (writes to Program Files)
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('elevating permissions to install hooks')
    localStorage.setItem('do-install-hooks', "true");
    await invoke('elevate_permissions')
    return;
  }

  await runFixJob('Download multiplayer fix', async () => {
    await installJobs.download_hooks.run();
  }, 'download-hooks');

  await runFixJob('Install multiplayer fix', async () => {
    await installJobs.unzip_hooks.run();
  });

  await runFixJob('Clean temp directory', async () => {
    await invoke('clean_temp_directory');
  });

  _isFixingHooks.value = false
  initSetupState()
}

// FIX LAA
const fixLAA = async () => {
  _isFixingLAA.value = true
  // Elevate permissions if needed (modifies wic.exe in Program Files)
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    console.log('elevating permissions to set LAA flag')
    localStorage.setItem('do-fix-laa', "true");
    await invoke('elevate_permissions')
    return;
  }

  await runFixJob('Set Large Address Aware flag', async () => {
    await installJobs.set_laa.run();
  });
  _isFixingLAA.value = false
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
              <p>
                Your patch level is <strong>{{ _gameVersion }}</strong>
              </p>
              <p>The edition is: <strong v-if="_isSoviet">Soviet Assault</strong><strong v-else>Vanilla</strong></p>
              <p>
                The version of your multiplayer fix is <strong>{{ _hooksVersion }}</strong>
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

        <div class="card mb-3" v-if="_needsPatch">
          <div class="card-header">Game patch required</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is not patched to the latest version (1.0.1.1). You need to install
              the
              latest patch to play online. The installation will download and install the patches automatically.
              <strong>PLEASE DON'T USE YOUR MOUSE AND KEYBOARD WHILE THE INSTALLATION RUNS</strong>
            </p>
            <button @click="installPatches" class="btn cta" :disabled="_isFixingPatches">Install patches</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsHooks">
          <div class="card-header">Enable online multiplayer</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is not correctly configured for multiplayer. You need to install
              the
              multiplayer fix.
            </p>
            <p>
              Your install directory is: {{ _installDir }}
            </p>
            <button @click="installHooks" class="btn cta" :disabled="_isFixingHooks">Install fix</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsLAA">
          <div class="card-header">Game patch required</div>
          <div class="card-body">
            <p>
              Your World in Conflict installation is not <em>Large Adress Aware</em>. This can cause problems when
              starting or running the game.
            </p>
            <button @click="fixLAA" class="btn cta" :disabled="_isFixingLAA">Fix problem</button>
          </div>
        </div>

        <div class="card mb-3" v-if="_needsHooksUpdate">
          <div class="card-header">Multiplayer fix</div>
          <div class="card-body">
            <p>
              Your World in Conflict multiplayer fix is outdated. You need to update it to the latest version.
            </p>
            <button @click="installHooks" class="btn cta" :disabled="_isFixingHooks">Update</button>
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
            <button class="btn cta" @click="setCDKey(_isSoviet ? SOVIET_KEY : VANILLA_KEY)"
              :disabled="_isFixingCDKey">Write CD Key to
              registry</button>
          </div>
          <div class="card-body" id="set-cdkey" v-if="CLAN_KEY || STEAM_KEY">
            <div id="set-cdkey-options">
              <div class="set-cdkey-option" v-if="CLAN_KEY">
                <div>Clan Edition<br />{{ CLAN_KEY }}</div>
                <button class="btn cta small primary" @click="setCDKey(CLAN_KEY)" :disabled="_isFixingCDKey">Write to
                  registry</button>
              </div>
              <div class="set-cdkey-option" v-if="STEAM_KEY">
                <div>Steam Edition<br />{{ STEAM_KEY }}</div>
                <button class="btn cta small primary" @click="setCDKey(STEAM_KEY)" :disabled="_isFixingCDKey">Write to
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
              <button class="btn cta" @click="installVCRedist" :disabled="_isFixingVCRedist">Install VC Redist</button>
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