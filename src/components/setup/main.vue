<script setup lang="ts">
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api';
import { onMounted, reactive, ref } from 'vue';

import jobsVue from '../jobs.vue'
import { Job, Job_Data } from '../../lib/wic-job';
import useInstallJobs from '../../lib/install-jobs'
import useSetupState from '../../lib/use-setup-state'

const installJobs = useInstallJobs();
const {
  initializeSuccess,
  needsCDKey: _needsCDKey,
  needsVCRedist: _needsVCRedist,
  needsPatch: _needsPatch,
  needsHooks: _needsHooks,
  needsHooksUpdate: _needsHooksUpdate,
  needsLAA: _needsLAA,
  needsAction: _needsAction,
  installDir: _installDir,
  isSoviet: _isSoviet,
  cdKey: _cdKey,
  initSetupState,
  ensureInit,
} = useSetupState();

const VANILLA_KEY = '3EXO-ELED-MXGY-FP5M-286R'
const SOVIET_KEY = 'LABG-U3MF-RG9G-95GB-AYTH'
const CLAN_KEY = ref(null)
const STEAM_KEY = ref(null)

const _jobsFixes: Job[] = reactive([])
const _display = ref(false)

// Track running state of fix actions
const _isFixingPatches = ref(false)
const _isFixingHooks = ref(false)
const _isFixingLAA = ref(false)
const _isFixingVCRedist = ref(false)
const _isFixingCDKey = ref(false)

const runFixJob = async (title, action, progressFilter: string | null = null) => {
  await createFixJob(title, action, progressFilter).run()
}
const createFixJob = (title, action, progressFilter: string | null = null) => {
  const data: Job_Data = reactive({
    title,
    status: 'queued',
    info: [],
    progress: 0
  })

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
    } finally {
      if (unlisten) unlisten()
      initSetupState()
    }
  }
  const job = { data, run }
  _jobsFixes.push(job)
  return job
}

onMounted(async () => {
  CLAN_KEY.value = await invoke('get_secret', { secret: 'SECRET_CLAN_KEY' })
  STEAM_KEY.value = await invoke('get_secret', { secret: 'SECRET_STEAM_KEY' })

  await ensureInit()

  if (_needsAction.value)
    _display.value = true;

  // If we're elevated, automatically run pending fixes
  const isElevated = await invoke('is_elevated')
  if (isElevated) {
    if (_needsHooks.value || _needsHooksUpdate.value) installHooks()
    if (_needsLAA.value) fixLAA()
    if (_needsVCRedist.value) installVCRedist()
  }
})

const installPatches = async () => {
  _isFixingPatches.value = true

  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
    await invoke('elevate_permissions')
    return;
  }

  const todo: Job[] = []

  let version = await invoke('extract_game_version') as any;
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
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
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

  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
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
  let isElevated = await invoke('is_elevated')
  if (!isElevated) {
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
  <div id="setup" class="mb-5" v-if="initializeSuccess && _display">
    <h2>Setup</h2>
    <div class="setup-flex">
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

.big-error {
  background: red;
  border-radius: 5px;
  padding: 10px;
  font-weight: bold;
  font-size: 1.2em;
}
</style>
