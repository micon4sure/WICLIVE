<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { openUrl } from '@tauri-apps/plugin-opener'

type ActionStatus = 'checking' | 'done' | 'needed' | 'applying' | 'fixed' | 'error'

interface Action {
  id: string
  label: string
  status: ActionStatus
  detail?: string
}

type AppMode = 'checking' | 'not-installed' | 'broken' | 'needs-fixes' | 'ready'

const props = defineProps<{
  actions: Action[]
  mode: AppMode
}>()

const emit = defineEmits<{
  fixed: []
}>()

const clearingRegistry = ref(false)
const registryCleared = ref(false)
const registryError = ref('')

async function clearRegistry() {
  clearingRegistry.value = true
  registryError.value = ''
  try {
    await invoke('clear_install_registry')
    registryCleared.value = true
  } catch (e) {
    registryError.value = String(e)
  }
  clearingRegistry.value = false
}

const fixing = ref(false)
const currentFix = ref('')
const currentStage = ref('')
const dlProgress = ref(0)
const dlTotal = ref(0)
const dlDone = ref(false)
const exProgress = ref(0)
const exTotal = ref(0)
let lastProgressUpdate = 0

const items: Record<string, { label: string; desc: string }> = {
  laa: {
    label: 'Large Address Aware',
    desc: 'Your game executable is not Large Address Aware. This can cause crashes when starting or running the game.',
  },
  vcredist: {
    label: 'VC++ Redistributable',
    desc: 'The Visual Studio C++ Redistributable is missing. This is required to run the game.',
  },
  patch: {
    label: 'Game Patches',
    desc: 'Your game version is outdated and needs to be patched to play online.',
  },
  cdkey: {
    label: 'CD Key',
    desc: 'No valid CD key is set. You need a CD key to play online.',
  },
  hooks: {
    label: 'Multiplayer Proxy',
    desc: 'The multiplayer proxy is not installed. This is required to connect to WiCGate servers.',
  },
}



const visibleActions = computed(() =>
  props.actions.filter(a => a.status !== 'checking' && a.status !== 'done')
)

const allFixed = computed(() =>
  visibleActions.value.length > 0 && !visibleActions.value.some(a => a.status === 'needed' || a.status === 'error' || a.status === 'applying')
)

function failedActions() {
  return props.actions.filter(a => a.status === 'needed' || a.status === 'error')
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const mb = bytes / (1024 * 1024)
  if (mb >= 1) return `${mb.toFixed(1)} MB`
  const kb = bytes / 1024
  return `${kb.toFixed(0)} KB`
}

function dlPercent(): number {
  if (dlTotal.value === 0) return 0
  return Math.round((dlProgress.value / dlTotal.value) * 100)
}

function exPercent(): number {
  if (exTotal.value === 0) return 0
  return Math.round((exProgress.value / exTotal.value) * 100)
}

async function runFixes() {
  fixing.value = true
  const failed = failedActions()

  // Listen for patch download progress
  const unlisten = await listen<{
    stage: string
    downloaded: number
    total: number
    detail: string
  }>('patch-progress', (event) => {
    const p = event.payload
    if (p.stage === 'downloading') {
      const now = Date.now()
      if (p.downloaded < p.total && now - lastProgressUpdate < 200) return
      lastProgressUpdate = now
      dlProgress.value = p.downloaded
      dlTotal.value = p.total
      if (p.downloaded >= p.total) dlDone.value = true
      currentStage.value = 'downloading'
    } else if (p.stage === 'extracting') {
      if (!dlDone.value) dlDone.value = true
      const now = Date.now()
      if (p.downloaded < p.total && now - lastProgressUpdate < 200) return
      lastProgressUpdate = now
      exProgress.value = p.downloaded
      exTotal.value = p.total
      currentStage.value = 'extracting'
    }
  })

  for (const action of failed) {
    currentFix.value = action.id
    action.status = 'applying'
    dlProgress.value = 0
    dlTotal.value = 0
    dlDone.value = false
    exProgress.value = 0
    exTotal.value = 0

    try {
      if (action.id === 'patch') {
        currentStage.value = 'downloading'
        action.detail = 'Downloading...'
        await invoke('apply_patches')
        action.status = 'fixed'
        action.detail = 'Patched to 1.0.1.1'
      } else if (action.id === 'laa') {
        action.detail = 'Applying...'
        await invoke('set_laa_flag')
        action.status = 'fixed'
        action.detail = 'Enabled'
      } else if (action.id === 'cdkey') {
        action.detail = 'Requesting key...'
        const key = await invoke<string>('request_cd_key')
        await invoke('set_cd_key', { key })
        action.status = 'fixed'
        action.detail = key
      } else if (action.id === 'hooks') {
        action.detail = 'Installing...'
        const ver = await invoke<string>('install_proxy')
        action.status = 'fixed'
        action.detail = ver || 'Installed'
      } else {
        action.status = 'needed'
        continue
      }
    } catch (e) {
      action.status = 'error'
      action.detail = String(e)
    }
  }

  unlisten()
  currentFix.value = ''
  fixing.value = false

  emit('fixed')
}

</script>

<template>
  <div class="readiness-card" :class="{ 'is-done': allFixed, 'is-fatal': mode === 'broken' || mode === 'not-installed' }">
    <!-- Not installed: no registry entry -->
    <div v-if="mode === 'not-installed'" class="install-missing">
      <div class="missing-icon">&#x25CB;</div>
      <div class="missing-text">
        <h3>Game Not Installed</h3>
        <p>World in Conflict is not installed on this computer.</p>
        <button class="btn btn-sm btn-get-game" @click="openUrl('https://www.gog.com/de/game/world_in_conflict_complete_edition')">
          Get it on GOG.com
        </button>
        <p class="missing-hint">Install the game and restart WIC LIVE.</p>
      </div>
    </div>

    <!-- Broken: registry exists but files missing -->
    <div v-else-if="mode === 'broken'" class="install-missing">
      <div class="missing-icon">&#x2717;</div>
      <div class="missing-text">
        <h3>Game Not Found</h3>
        <p>The Windows registry says the game is installed, but the files are not there. This usually happens when the game was deleted without being properly uninstalled.</p>
        <p class="missing-hint">You can clean up the stale registry entry, then reinstall the game.</p>
        <button class="btn btn-sm btn-fix-registry" :disabled="clearingRegistry" @click="clearRegistry">
          {{ clearingRegistry ? 'Cleaning...' : 'Remove from Registry' }}
        </button>
        <span v-if="registryCleared" class="registry-cleared">Done — reinstall the game and restart WIC LIVE.</span>
        <span v-if="registryError" class="registry-error">{{ registryError }}</span>
      </div>
    </div>

    <!-- Checking -->
    <div v-else-if="mode === 'checking'" class="readiness-header">
      <h3 class="header-title">
        <span class="title-text title-checking">Checking...</span>
      </h3>
    </div>

    <!-- Needs fixes -->
    <template v-else>
    <div class="readiness-header">
      <div class="readiness-header-row">
        <h3 class="header-title">
          <span class="title-text title-pending" :class="{ 'title-hidden': allFixed }">Game Readiness</span>
          <span class="title-text title-done" :class="{ 'title-hidden': !allFixed }">Game Readiness: All Set</span>
        </h3>
        <div class="header-btn-wrap" :class="{ 'btn-hidden': allFixed }">
          <button class="btn btn-sm btn-primary" :disabled="fixing" @click="runFixes">Fix</button>
        </div>
      </div>
      <div class="header-sub" :class="{ 'sub-hidden': allFixed }">
        <p v-if="fixing">Fixing issues...</p>
        <p v-else>The following issues need to be resolved before you can play.</p>
      </div>
    </div>
    <div class="readiness-body">
      <div
        v-for="action in visibleActions"
        :key="action.id"
        class="readiness-item"
        :class="['status-' + action.status, { 'item-collapsed': action.status === 'fixed' }]"
      >
        <div class="item-row">
          <span class="item-label">{{ items[action.id]?.label || action.label }}</span>
          <span class="item-status">
            <template v-if="action.status === 'applying'">fixing...</template>
            <template v-else-if="action.status === 'fixed'">fixed</template>
            <template v-else-if="action.status === 'done'">ok</template>
            <template v-else-if="action.status === 'error'">failed</template>
            <template v-else>pending</template>
          </span>
        </div>

        <div class="item-detail-wrap">
          <span v-if="action.status === 'fixed'" class="item-detail item-fixed">
            {{ action.detail }}
          </span>
          <span v-else-if="action.status !== 'applying' && action.status !== 'done' && action.status !== 'fixed'" class="item-desc">
            {{ items[action.id]?.desc }}
          </span>
        </div>

        <!-- Download progress -->
        <div v-if="action.status === 'applying' && currentFix === action.id && dlTotal > 0" class="progress-area">
          <div class="progress-stage">
            <span class="stage-icon" :class="{ 'stage-done': dlDone }">{{ dlDone ? '&#x2713;' : '' }}</span>
            <span class="stage-label">Download</span>
          </div>
          <div v-if="!dlDone" class="progress-track">
            <div class="progress-fill progress-fill-dl" :style="{ width: dlPercent() + '%' }" />
            <span class="progress-label">{{ dlPercent() }}%</span>
          </div>
          <div v-if="!dlDone" class="progress-meta">
            <span>{{ formatBytes(dlProgress) }} / {{ formatBytes(dlTotal) }}</span>
            <span>Downloading...</span>
          </div>
        </div>

        <!-- Extract progress -->
        <div v-if="action.status === 'applying' && currentFix === action.id && dlDone" class="progress-area">
          <div class="progress-stage">
            <span class="stage-icon" :class="{ 'stage-done': exProgress >= exTotal && exTotal > 0 }">{{ exProgress >= exTotal && exTotal > 0 ? '&#x2713;' : '' }}</span>
            <span class="stage-label">Extract</span>
          </div>
          <div v-if="exTotal > 0" class="progress-track">
            <div class="progress-fill progress-fill-ex" :style="{ width: exPercent() + '%' }" />
            <span class="progress-label">{{ exPercent() }}%</span>
          </div>
          <div v-if="exTotal > 0" class="progress-meta">
            <span>{{ exProgress }} / {{ exTotal }} files</span>
            <span>Extracting...</span>
          </div>
        </div>

        <span v-if="action.status === 'applying' && currentFix === action.id && dlTotal === 0" class="item-detail">
          {{ action.detail }}
        </span>

        <span v-if="action.status === 'error'" class="item-detail item-error">{{ action.detail }}</span>
      </div>
    </div>
    </template>
  </div>
</template>

<style scoped>
/* ── Card shell ──────────────────────────────────────── */
.readiness-card {
  border: 1px solid rgba(var(--sw-rgb), 0.3);
  background: rgba(var(--bg-rgb), 0.85);
  transition: border-color 0.6s ease;
}
.is-done { border-color: rgba(var(--g-rgb), 0.3); }
.is-fatal { border-color: rgba(var(--dl-light-rgb), 0.5); }

/* ── Install missing banner ────────────────────────────── */
.install-missing {
  display: flex;
  gap: 16px;
  padding: 20px 24px;
  align-items: flex-start;
}
.missing-icon {
  font-size: 28px;
  color: var(--dl-light);
  line-height: 1;
  flex-shrink: 0;
}
.missing-text h3 {
  margin: 0 0 6px;
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--dl-light);
}
.missing-text p {
  margin: 0 0 4px;
  font-size: 14px;
  color: var(--t2);
  line-height: 1.4;
}
.missing-hint {
  color: var(--t3) !important;
  font-size: 13px !important;
}
.btn-get-game {
  display: inline-block;
  font-family: 'Oswald', sans-serif;
  font-size: 14px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 8px 20px;
  margin: 12px 0;
  background: rgba(var(--sw-rgb), 0.2);
  border: 1px solid rgba(var(--sw-rgb), 0.5);
  color: var(--sw);
  cursor: pointer;
  transition: background 0.2s ease;
}
.btn-get-game:hover {
  background: rgba(var(--sw-rgb), 0.35);
}
.btn-fix-registry {
  font-family: 'Oswald', sans-serif;
  font-size: 13px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 6px 16px;
  background: rgba(var(--dl-light-rgb), 0.2);
  border: 1px solid rgba(var(--dl-light-rgb), 0.4);
  color: var(--dl-light);
  cursor: pointer;
  transition: background 0.2s ease;
}
.btn-fix-registry:hover:not(:disabled) {
  background: rgba(var(--dl-light-rgb), 0.35);
}
.btn-fix-registry:disabled {
  opacity: 0.5;
  cursor: default;
}
.registry-cleared {
  display: block;
  margin-top: 10px;
  font-size: 14px;
  color: var(--g);
}
.registry-error {
  display: block;
  margin-top: 10px;
  font-size: 13px;
  color: var(--dl-light);
}

/* ── Header ──────────────────────────────────────────── */
.readiness-header {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(var(--sw-rgb), 0.2);
  transition: padding 0.5s ease, border-color 0.6s ease;
}
.is-done .readiness-header {
  padding: 10px 20px;
  border-color: rgba(var(--g-rgb), 0.2);
}

.readiness-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-title {
  position: relative;
  margin: 0 0 4px;
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.title-text {
  transition: opacity 0.4s ease, color 0.4s ease;
}
.title-checking { color: var(--t3); }
.title-pending {
  color: var(--sw);
  position: absolute;
  left: 0;
  top: 0;
}
.title-done { color: var(--g); }
.title-hidden { opacity: 0; pointer-events: none; }

.header-btn-wrap {
  overflow: hidden;
  max-width: 120px;
  opacity: 1;
  transition: max-width 0.4s ease, opacity 0.3s ease;
}
.header-btn-wrap.btn-hidden {
  max-width: 0;
  opacity: 0;
}

.header-sub {
  max-height: 30px;
  opacity: 1;
  overflow: hidden;
  transition: max-height 0.4s ease, opacity 0.3s ease;
}
.header-sub.sub-hidden {
  max-height: 0;
  opacity: 0;
}
.header-sub p {
  margin: 0;
  font-size: 15px;
  color: var(--t2);
}

/* ── Body ────────────────────────────────────────────── */
.readiness-body {
  padding: 12px 20px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  transition: padding 0.5s ease, gap 0.5s ease;
}
.is-done .readiness-body {
  padding: 6px 20px;
  gap: 2px;
}

/* ── Items ────────────────────────────────────────────── */
.readiness-item {
  padding: 12px 16px;
  border-left: 3px solid var(--t3);
  background: rgba(255, 255, 255, 0.03);
  transition: padding 0.5s ease, border-color 0.4s ease;
}
.is-done .readiness-item,
.item-collapsed {
  padding: 4px 16px;
  background: transparent;
}

.item-collapsed .item-row { margin-bottom: 0; }
.item-collapsed .item-label { font-size: 12px; }
.item-collapsed .item-status { font-size: 11px; }
.item-collapsed .item-detail-wrap {
  max-height: 0;
  opacity: 0;
}

.item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
  transition: margin 0.5s ease;
}
.is-done .item-row { margin-bottom: 0; }

.item-label {
  font-family: 'Oswald', sans-serif;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-size: 15px;
  color: #fff;
  transition: font-size 0.5s ease, color 0.4s ease;
}
.is-done .item-label { font-size: 12px; }

.item-status {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--t3);
  transition: font-size 0.5s ease, color 0.4s ease;
}
.is-done .item-status { font-size: 11px; }

.item-detail-wrap {
  max-height: 60px;
  opacity: 1;
  overflow: hidden;
  transition: max-height 0.5s ease, opacity 0.4s ease;
}
.is-done .item-detail-wrap {
  max-height: 0;
  opacity: 0;
}

/* ── Status colors ───────────────────────────────────── */
.status-needed  { border-left-color: var(--sw); }
.status-applying { border-left-color: var(--b); }
.status-done    { border-left-color: var(--t3); opacity: 0.6; }
.status-fixed   { border-left-color: var(--g); }
.status-error   { border-left-color: var(--dl-light); }

.status-applying .item-status { color: var(--b); }
.status-done .item-status     { color: var(--t3); }
.status-fixed .item-status    { color: var(--g); }
.status-fixed .item-label     { color: var(--g); }

/* ── Detail text ─────────────────────────────────────── */
.item-desc {
  display: block;
  font-size: 14px;
  color: var(--t2);
  line-height: 1.4;
}

.item-detail {
  display: block;
  margin-top: 4px;
  font-size: 13px;
  color: var(--t3);
}
.item-fixed { color: var(--g); }
.item-error { color: var(--dl-light); }

/* ── Progress bar ────────────────────────────────────── */
.progress-area { margin-top: 8px; }

.progress-stage {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.stage-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  font-size: 10px;
  font-weight: 700;
  border: 1px solid var(--t3);
  color: var(--t3);
  transition: border-color 0.3s, color 0.3s;
}

.stage-icon.stage-done {
  border-color: var(--g);
  color: var(--g);
}

.stage-label {
  font-family: 'Rajdhani', sans-serif;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--t2);
}

.progress-track {
  position: relative;
  height: 22px;
  background: rgba(var(--mg-rgb), 0.5);
  border: 1px solid rgba(var(--mg-rgb), 0.6);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-size: 22.6px 100%, 100% 100%;
  transition: width 0.15s linear;
  animation: progress-stripes 0.6s linear infinite;
  position: relative;
}

.progress-fill-dl {
  background:
    repeating-linear-gradient(-45deg, transparent, transparent 8px, rgba(255,255,255,0.08) 8px, rgba(255,255,255,0.08) 16px),
    linear-gradient(180deg, rgba(var(--b-rgb), 0.95) 0%, rgba(var(--b-rgb), 0.7) 100%);
  background-size: 22.6px 100%, 100% 100%;
  box-shadow: 0 0 12px rgba(var(--b-rgb), 0.3);
}

.progress-fill-ex {
  background:
    repeating-linear-gradient(-45deg, transparent, transparent 8px, rgba(255,255,255,0.08) 8px, rgba(255,255,255,0.08) 16px),
    linear-gradient(180deg, rgba(var(--b-rgb), 0.95) 0%, rgba(var(--b-rgb), 0.7) 100%);
  background-size: 22.6px 100%, 100% 100%;
  box-shadow: 0 0 12px rgba(var(--b-rgb), 0.3);
}

.progress-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.15) 50%, transparent 100%);
  background-size: 200% 100%;
  animation: progress-shimmer 2s ease-in-out infinite;
}

@keyframes progress-stripes {
  from { background-position: 0 0, 0 0; }
  to { background-position: 22.6px 0, 0 0; }
}
@keyframes progress-shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.progress-label {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Oswald', sans-serif;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #fff;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
}

.progress-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  color: var(--t3);
}
</style>
