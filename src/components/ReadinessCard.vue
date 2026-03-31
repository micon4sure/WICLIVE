<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

type ActionStatus = 'checking' | 'done' | 'needed' | 'applying' | 'fixed' | 'error'

interface Action {
  id: string
  label: string
  status: ActionStatus
  detail?: string
}

const props = defineProps<{
  actions: Action[]
}>()

const emit = defineEmits<{
  fixed: []
}>()

const fixing = ref(false)
const currentFix = ref('')
const currentStage = ref('')
const downloadProgress = ref(0)
const downloadTotal = ref(0)
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
    desc: 'The multiplayer proxy is not installed. This is required to connect to WIC LIVE servers.',
  },
}

const collapsed = ref(false)

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

function progressPercent(): number {
  if (downloadTotal.value === 0) return 0
  return Math.round((downloadProgress.value / downloadTotal.value) * 100)
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
    if (p.stage === 'downloading' || p.stage === 'extracting') {
      const now = Date.now()
      if (now - lastProgressUpdate < 200) return
      lastProgressUpdate = now
      downloadProgress.value = p.downloaded
      downloadTotal.value = p.total
      currentStage.value = p.stage
    }
  })

  for (const action of failed) {
    currentFix.value = action.id
    action.status = 'applying'
    downloadProgress.value = 0
    downloadTotal.value = 0

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

  if (allFixed.value) {
    await nextTick()
    setTimeout(() => { collapsed.value = true }, 600)
  }

  emit('fixed')
}

</script>

<template>
  <div class="readiness-card" :class="{ 'is-collapsed': collapsed }">
    <div class="readiness-header">
      <div class="readiness-header-row">
        <h3 class="header-title">
          <span class="title-text title-pending" :class="{ 'title-hidden': collapsed }">Game Readiness</span>
          <span class="title-text title-done" :class="{ 'title-hidden': !collapsed }">All Good</span>
        </h3>
        <div class="header-btn-wrap" :class="{ 'btn-hidden': allFixed || collapsed }">
          <button class="btn btn-sm btn-primary" :disabled="fixing" @click="runFixes">Fix</button>
        </div>
      </div>
      <div class="header-sub" :class="{ 'sub-hidden': allFixed || collapsed }">
        <p v-if="fixing">Fixing issues...</p>
        <p v-else>The following issues need to be resolved before you can play.</p>
      </div>
    </div>
    <div class="readiness-body">
      <div
        v-for="action in visibleActions"
        :key="action.id"
        class="readiness-item"
        :class="'status-' + action.status"
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

        <!-- Progress bar (download + extract) -->
        <div v-if="action.status === 'applying' && currentFix === action.id && downloadTotal > 0" class="progress-area">
          <div class="progress-track">
            <div class="progress-fill" :style="{ width: progressPercent() + '%' }" />
            <span class="progress-label">{{ progressPercent() }}%</span>
          </div>
          <div class="progress-meta">
            <span v-if="currentStage === 'downloading'">{{ formatBytes(downloadProgress) }} / {{ formatBytes(downloadTotal) }}</span>
            <span v-else>{{ downloadProgress }} / {{ downloadTotal }} files</span>
            <span>{{ currentStage === 'downloading' ? 'Downloading...' : 'Extracting...' }}</span>
          </div>
        </div>

        <span v-if="action.status === 'applying' && currentFix === action.id && downloadTotal === 0" class="item-detail">
          {{ action.detail }}
        </span>

        <span v-if="action.status === 'error'" class="item-detail item-error">{{ action.detail }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── Card shell ──────────────────────────────────────── */
.readiness-card {
  border: 1px solid rgba(var(--sw-rgb), 0.3);
  transition: border-color 0.6s ease;
}
.is-collapsed { border-color: rgba(var(--g-rgb), 0.3); }

/* ── Header ──────────────────────────────────────────── */
.readiness-header {
  padding: 16px 20px;
  border-bottom: 1px solid rgba(var(--sw-rgb), 0.2);
  transition: padding 0.5s ease, border-color 0.6s ease;
}
.is-collapsed .readiness-header {
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
.title-pending { color: var(--sw); }
.title-done {
  color: var(--g);
  position: absolute;
  left: 0;
  top: 0;
}
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
.is-collapsed .readiness-body {
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
.is-collapsed .readiness-item {
  padding: 4px 16px;
  background: transparent;
}

.item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
  transition: margin 0.5s ease;
}
.is-collapsed .item-row { margin-bottom: 0; }

.item-label {
  font-family: 'Oswald', sans-serif;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-size: 15px;
  color: #fff;
  transition: font-size 0.5s ease, color 0.4s ease;
}
.is-collapsed .item-label { font-size: 12px; }

.item-status {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--t3);
  transition: font-size 0.5s ease, color 0.4s ease;
}
.is-collapsed .item-status { font-size: 11px; }

.item-detail-wrap {
  max-height: 60px;
  opacity: 1;
  overflow: hidden;
  transition: max-height 0.5s ease, opacity 0.4s ease;
}
.is-collapsed .item-detail-wrap {
  max-height: 0;
  opacity: 0;
}

/* ── Status colors ───────────────────────────────────── */
.status-needed  { border-left-color: var(--sw); }
.status-applying { border-left-color: var(--sw); }
.status-done    { border-left-color: var(--t3); opacity: 0.6; }
.status-fixed   { border-left-color: var(--g); }
.status-error   { border-left-color: var(--dl-light); }

.status-applying .item-status { color: var(--sw); }
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
.progress-area { margin-top: 10px; }

.progress-track {
  position: relative;
  height: 22px;
  background: rgba(var(--mg-rgb), 0.5);
  border: 1px solid rgba(var(--mg-rgb), 0.6);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background:
    repeating-linear-gradient(-45deg, transparent, transparent 8px, rgba(255,255,255,0.08) 8px, rgba(255,255,255,0.08) 16px),
    linear-gradient(180deg, rgba(var(--sw-rgb), 0.95) 0%, rgba(var(--sw-rgb), 0.7) 100%);
  background-size: 22.6px 100%, 100% 100%;
  box-shadow: 0 0 12px rgba(var(--sw-rgb), 0.3);
  transition: width 0.15s linear;
  animation: progress-stripes 0.6s linear infinite;
  position: relative;
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
