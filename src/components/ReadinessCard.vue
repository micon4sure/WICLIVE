<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useGameState } from '../composables/useGameState'

const { readinessActions, wasFixed, check } = useGameState()

const fixing = ref(false)
const currentFix = ref('')
const currentStage = ref('')
const dlProgress = ref(0)
const dlTotal = ref(0)
const dlDone = ref(false)
const exProgress = ref(0)
const exTotal = ref(0)
let lastProgressUpdate = 0

const fixStatus = ref<Record<string, 'applying' | 'fixed' | 'error'>>({})
const fixDetail = ref<Record<string, string>>({})

const meta: Record<string, { label: string; desc: string }> = {
  vcredist: {
    label: 'VC++ Redistributable',
    desc: 'The Visual Studio C++ Redistributable is missing. This is required to run the game.',
  },
  dx9: {
    label: 'DirectX 9 Runtime',
    desc: 'The DirectX 9 June 2010 runtime is missing. This is required to run the game.',
  },
  patch: {
    label: 'Game Patches',
    desc: 'Your game version is outdated and needs to be patched to play online.',
  },
  laa: {
    label: 'Large Address Aware',
    desc: 'Your game executable is not Large Address Aware. This can cause crashes when starting or running the game.',
  },
  cdkey: {
    label: 'CD Key',
    desc: 'No valid CD key is set. You need a CD key to play online.',
  },
  proxy_installed: {
    label: 'Multiplayer Proxy',
    desc: 'The multiplayer proxy is not installed. This is required to connect to WiCGate servers.',
  },
  proxy_current: {
    label: 'Proxy Update',
    desc: 'A newer version of the multiplayer proxy is available.',
  },
}

const visibleItems = computed(() =>
  Object.entries(readinessActions.value)
    .filter(([id, a]) => (a.need && !a.has) || fixStatus.value[id])
    .map(([id, a]) => ({
      id,
      label: meta[id]?.label || id,
      desc: meta[id]?.desc || '',
      status: fixStatus.value[id] || 'needed',
      detail: fixDetail.value[id] || a.detail,
    }))
)

const allFixed = computed(() =>
  visibleItems.value.length > 0 && visibleItems.value.every(i => i.status === 'fixed')
)

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

  const toFix = Object.entries(readinessActions.value)
    .filter(([, a]) => a.need && !a.has)
    .map(([id]) => id)

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

  for (const id of toFix) {
    currentFix.value = id
    fixStatus.value[id] = 'applying'
    dlProgress.value = 0
    dlTotal.value = 0
    dlDone.value = false
    exProgress.value = 0
    exTotal.value = 0

    try {
      if (id === 'patch') {
        currentStage.value = 'downloading'
        fixDetail.value[id] = 'Downloading...'
        await invoke('apply_patches')
        fixStatus.value[id] = 'fixed'
        fixDetail.value[id] = 'Patched to 1.0.1.1'
      } else if (id === 'laa') {
        fixDetail.value[id] = 'Applying...'
        await invoke('set_laa_flag')
        fixStatus.value[id] = 'fixed'
        fixDetail.value[id] = 'Enabled'
      } else if (id === 'cdkey') {
        fixDetail.value[id] = 'Requesting key...'
        const key = await invoke<string>('request_cd_key', { source: 'existing' })
        await invoke('set_cd_key', { key })
        fixStatus.value[id] = 'fixed'
        fixDetail.value[id] = key
      } else if (id === 'dx9') {
        currentStage.value = 'downloading'
        fixDetail.value[id] = 'Downloading...'
        await invoke('install_dx9')
        fixStatus.value[id] = 'fixed'
        fixDetail.value[id] = 'Installed'
      } else if (id === 'proxy_installed' || id === 'proxy_current') {
        fixDetail.value[id] = id === 'proxy_current' ? 'Updating...' : 'Installing...'
        const ver = await invoke<string>('install_proxy')
        fixStatus.value[id] = 'fixed'
        fixDetail.value[id] = ver || 'Installed'
      } else {
        delete fixStatus.value[id]
        continue
      }
    } catch (e) {
      fixStatus.value[id] = 'error'
      fixDetail.value[id] = String(e)
    }
  }

  unlisten()
  currentFix.value = ''
  fixing.value = false

  // Set before check() so the card doesn't unmount during re-check
  wasFixed.value = toFix.every(id => fixStatus.value[id] === 'fixed')

  await check()

  // Downgrade if check reveals remaining issues
  if (Object.values(readinessActions.value).some(a => a.need && !a.has)) {
    wasFixed.value = false
  }
}
</script>

<template>
  <div class="action-card readiness-card" :class="{ 'is-done': allFixed, 'is-fixing': fixing }">
    <div class="readiness-header">
      <div class="readiness-header-row">
        <h3 class="header-title">
          <span class="title-text title-pending" :class="{ 'title-hidden': allFixed || fixing }">Game Readiness</span>
          <span class="title-text title-fixing" :class="{ 'title-hidden': !fixing || allFixed }">Fixing Issues</span>
          <span class="title-text title-done" :class="{ 'title-hidden': !allFixed }">Game Readiness: All Set</span>
        </h3>
        <div class="header-btn-wrap" :class="{ 'btn-hidden': allFixed }">
          <button class="btn btn-primary" :disabled="fixing" @click="runFixes">Fix</button>
        </div>
      </div>
      <div class="header-sub" :class="{ 'sub-hidden': allFixed }">
        <p v-if="!fixing">The following issues need to be resolved before you can play.</p>
      </div>
    </div>
    <div class="readiness-body">
      <div v-for="item in visibleItems" :key="item.id" class="readiness-item"
        :class="['status-' + item.status, { 'item-collapsed': item.status === 'fixed' }]">
        <div class="item-row">
          <span class="item-label">{{ item.label }}</span>
          <span class="item-status">
            <template v-if="item.status === 'applying'">fixing...</template>
            <template v-else-if="item.status === 'fixed'">fixed</template>
            <template v-else-if="item.status === 'error'">failed</template>
            <template v-else>pending</template>
          </span>
        </div>

        <div class="item-detail-wrap">
          <span v-if="item.status === 'fixed'" class="item-detail item-fixed">
            {{ item.detail }}
          </span>
          <span v-else-if="item.status !== 'applying'" class="item-desc">
            {{ item.desc }}
          </span>
        </div>

        <!-- Download progress -->
        <div v-if="item.status === 'applying' && currentFix === item.id && dlTotal > 0" class="progress-area">
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
        <div v-if="item.status === 'applying' && currentFix === item.id && dlDone" class="progress-area">
          <div class="progress-stage">
            <span class="stage-icon" :class="{ 'stage-done': exProgress >= exTotal && exTotal > 0 }">{{ exProgress >=
              exTotal &&
              exTotal > 0 ? '&#x2713;' : '' }}</span>
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

        <span v-if="item.status === 'applying' && currentFix === item.id && dlTotal === 0" class="item-detail">
          {{ item.detail }}
        </span>

        <span v-if="item.status === 'error'" class="item-detail item-error">{{ item.detail }}</span>
      </div>
    </div>
  </div>
</template>

<style src="../assets/styles/card.css"></style>
<style scoped>
.readiness-card {
  border-color: rgba(var(--c-pending-rgb), 0.5);
}

.readiness-card .readiness-header {
  border-bottom-color: rgba(var(--c-pending-rgb), 0.3);
}

.readiness-card.is-fixing {
  border-color: rgba(var(--c-progress-rgb), 0.5);
}

.readiness-card.is-fixing .readiness-header {
  border-bottom-color: rgba(var(--c-progress-rgb), 0.3);
}

.readiness-card.is-done {
  border-color: rgba(var(--c-success-rgb), 0.3);
}

.readiness-card.is-done .readiness-header {
  border-bottom-color: rgba(var(--c-success-rgb), 0.2);
}

.title-pending {
  color: var(--c-pending);
}

.title-fixing {
  color: var(--c-progress);
  position: absolute;
  left: 0;
  top: 0;
}

.readiness-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-title {
  position: relative;
}

.title-pending,
.title-fixing {
  position: absolute;
  left: 0;
  top: 0;
}

.title-hidden {
  opacity: 0;
  pointer-events: none;
}

.header-btn-wrap {
  max-width: 120px;
  opacity: 1;
  transition: max-width 0.4s ease, opacity 0.3s ease;
}

.header-btn-wrap.btn-hidden {
  max-width: 0;
  opacity: 0;
  overflow: hidden;
}

.header-sub.sub-hidden {
  max-height: 0;
  opacity: 0;
}

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
  border: 1px solid var(--text-tertiary);
  color: var(--text-tertiary);
  transition: border-color 0.3s, color 0.3s;
}

.stage-icon.stage-done {
  border-color: var(--c-success);
  color: var(--c-success);
}

.stage-label {
  font-family: 'Rajdhani', sans-serif;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
}
</style>
