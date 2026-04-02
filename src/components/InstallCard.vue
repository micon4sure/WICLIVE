<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { openUrl } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'
import { useGameState } from '../composables/useGameState'

const { onInstalled } = useGameState()

const downloading = ref(false)
const dlDone = ref(false)
const extracting = ref(false)
const extractDone = ref(false)
const cdKey = ref('')
const cdKeyError = ref('')
const cdKeyDone = ref(false)
const error = ref('')
const dlProgress = ref(0)
const dlTotal = ref(0)
const exProgress = ref(0)
const exTotal = ref(0)
const installDir = ref('')
const installSubdir = ref('World in Conflict')
const installDirSet = ref(false)

const fullInstallPath = computed(() => {
  if (!installDir.value) return ''
  const sub = installSubdir.value.trim()
  return sub ? `${installDir.value}\\${sub}` : installDir.value
})

function dlPercent(): number {
  if (dlTotal.value === 0) return 0
  return Math.round((dlProgress.value / dlTotal.value) * 100)
}

function exPercent(): number {
  if (exTotal.value === 0) return 0
  return Math.round((exProgress.value / exTotal.value) * 100)
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const mb = bytes / (1024 * 1024)
  if (mb >= 1) return `${mb.toFixed(1)} MB`
  const kb = bytes / 1024
  return `${kb.toFixed(0)} KB`
}

async function pickInstallDir() {
  const selected = await open({ directory: true, title: 'Choose install directory' })
  if (selected) installDir.value = selected as string
}

function confirmInstallDir() {
  installDirSet.value = true
  maybeExtract()
}

async function maybeExtract() {
  if (!dlDone.value || !installDirSet.value || extracting.value) return
  extracting.value = true

  const unlisten = await listen<{
    stage: string
    downloaded: number
    total: number
    detail: string
  }>('installer-progress', (event) => {
    const p = event.payload
    if (p.stage === 'extracting') {
      exProgress.value = p.downloaded
      exTotal.value = p.total
    } else if (p.stage === 'done') {
      extractDone.value = true
    }
  })

  try {
    await invoke('extract_installer', { installDir: fullInstallPath.value })
  } catch (e) {
    error.value = String(e)
    unlisten()
    extracting.value = false
    return
  }

  unlisten()
  extracting.value = false

  if (!extractDone.value) {
    extractDone.value = true
  }

  // Request and set CD key
  try {
    const key = await invoke<string>('request_cd_key')
    await invoke('set_cd_key', { key })
    cdKey.value = key
    cdKeyDone.value = true
    setTimeout(() => onInstalled(), 1000)
  } catch (e) {
    cdKeyError.value = String(e)
    setTimeout(() => onInstalled(), 1000)
  }
}

async function startDownload() {
  downloading.value = true
  error.value = ''
  dlProgress.value = 0
  dlTotal.value = 0
  dlDone.value = false
  let lastUpdate = 0

  const unlisten = await listen<{
    stage: string
    downloaded: number
    total: number
    detail: string
  }>('installer-progress', (event) => {
    const p = event.payload
    if (p.stage === 'downloading') {
      const now = Date.now()
      if (p.downloaded < p.total && now - lastUpdate < 200) return
      lastUpdate = now
      dlProgress.value = p.downloaded
      dlTotal.value = p.total
    } else if (p.stage === 'done') {
      dlDone.value = true
      maybeExtract()
    }
  })

  try {
    await invoke('download_installer')
  } catch (e) {
    error.value = String(e)
  }

  unlisten()

  if (!dlDone.value && !error.value) {
    dlDone.value = true
    maybeExtract()
  }
}
</script>

<template>
  <div class="action-card" :class="{ 'is-done': cdKeyDone, 'is-installing': downloading || dlDone, 'is-missing': !downloading && !dlDone && !cdKeyDone }">
    <div class="readiness-header">
      <h3 class="header-title">
        <span class="title-text" :class="cdKeyDone ? 'title-done' : (downloading || dlDone) ? 'title-install' : 'title-missing'">
          {{ cdKeyDone ? 'Installation Complete' : (downloading || dlDone) ? 'Installing' : 'Game Not Installed' }}
        </span>
      </h3>
      <div v-if="!downloading && !dlDone" class="header-sub">
        <p>World in Conflict is not installed on this computer.</p>
      </div>
    </div>

    <div v-if="downloading || dlDone" class="readiness-body">
      <!-- Download -->
      <div class="readiness-item" :class="[dlDone ? 'status-fixed' : 'status-applying', { 'item-collapsed': dlDone }]">
        <div class="item-row">
          <span class="item-label">Download</span>
          <span class="item-status">{{ dlDone ? 'done' : 'downloading...' }}</span>
        </div>
        <div class="item-detail-wrap">
          <span v-if="dlDone" class="item-detail item-fixed">{{ formatBytes(dlTotal) }}</span>
        </div>
        <div v-if="!dlDone && dlTotal > 0" class="progress-area">
          <div class="progress-track">
            <div class="progress-fill progress-fill-dl" :style="{ width: dlPercent() + '%' }" />
            <span class="progress-label">{{ dlPercent() }}%</span>
          </div>
          <div class="progress-meta">
            <span>{{ formatBytes(dlProgress) }} / {{ formatBytes(dlTotal) }}</span>
          </div>
        </div>
      </div>

      <!-- Install directory picker -->
      <div v-if="!extractDone && !extracting" class="readiness-item status-needed">
        <div class="item-row">
          <span class="item-label">Install Location</span>
          <span class="item-status">pending</span>
        </div>
        <div class="install-dir-picker">
          <div class="dir-row">
            <input class="dir-input dir-input-base" :value="installDir" readonly placeholder="Choose a directory..."
              @click="pickInstallDir" />
            <span class="dir-sep">\</span>
            <input class="dir-input dir-input-sub" v-model="installSubdir" placeholder="Subfolder" />
            <button class="btn btn-sm btn-browse" @click="pickInstallDir">Browse</button>
          </div>
          <div v-if="installDir" class="install-confirm-row">
            <button class="btn btn-sm btn-blue btn-install-confirm" :disabled="installDirSet"
              @click="confirmInstallDir">Install</button>
            <span class="install-path">{{ fullInstallPath }}</span>
          </div>
          <p v-if="installDirSet && !dlDone" class="missing-hint">Waiting for download to finish...</p>
        </div>
      </div>

      <!-- Extract -->
      <div v-if="extracting || extractDone" class="readiness-item"
        :class="[extractDone ? 'status-fixed' : 'status-applying', { 'item-collapsed': extractDone }]">
        <div class="item-row">
          <span class="item-label">Extract</span>
          <span class="item-status">{{ extractDone ? 'done' : 'extracting...' }}</span>
        </div>
        <div class="item-detail-wrap">
          <span v-if="extractDone" class="item-detail item-fixed">{{ fullInstallPath }}</span>
        </div>
        <div v-if="!extractDone && exTotal > 0" class="progress-area">
          <div class="progress-track">
            <div class="progress-fill progress-fill-ex" :style="{ width: exPercent() + '%' }" />
            <span class="progress-label">{{ exPercent() }}%</span>
          </div>
          <div class="progress-meta">
            <span>{{ exProgress }} / {{ exTotal }} files</span>
          </div>
        </div>
      </div>

      <!-- CD Key -->
      <div v-if="extractDone" class="readiness-item"
        :class="[cdKeyDone ? 'status-fixed' : cdKeyError ? 'status-error' : 'status-applying', { 'item-collapsed': cdKeyDone }]">
        <div class="item-row">
          <span class="item-label">CD Key</span>
          <span class="item-status">{{ cdKeyDone ? 'done' : cdKeyError ? 'failed' : 'requesting...' }}</span>
        </div>
        <div class="item-detail-wrap">
          <span v-if="cdKeyDone" class="item-detail item-fixed">{{ cdKey }}</span>
          <span v-else-if="cdKeyError" class="item-detail item-error">{{ cdKeyError }}</span>
        </div>
      </div>

      <p v-if="error" class="registry-error" style="padding: 8px 16px;">{{ error }}</p>
    </div>

    <div v-if="!downloading && !dlDone" class="readiness-body">
      <div class="missing-buttons">
        <button class="btn btn-sm btn-gold"
          @click="openUrl('https://www.gog.com/de/game/world_in_conflict_complete_edition')">
          Get it on GOG.com
        </button>
        <button class="btn btn-sm btn-blue" @click="startDownload">
          Install Multiplayer Only
        </button>
      </div>
    </div>
  </div>
</template>

<style src="../assets/styles/card.css"></style>
<style scoped>
.is-missing {
  border-color: rgba(var(--dl-light-rgb), 0.5);
}

.is-missing .readiness-header {
  border-bottom-color: rgba(var(--dl-light-rgb), 0.3);
}

.is-installing {
  border-color: rgba(var(--b-rgb), 0.5);
}

.is-installing .readiness-header {
  border-bottom-color: rgba(var(--b-rgb), 0.3);
}

.is-done {
  border-color: rgba(var(--g-rgb), 0.3);
}

.is-done .readiness-header {
  border-bottom-color: rgba(var(--g-rgb), 0.2);
}

.title-missing {
  color: var(--dl-light);
}

.title-install {
  color: var(--b);
}

.missing-hint {
  color: var(--t3) !important;
  font-size: 13px !important;
}

.install-dir-picker {
  margin-top: 12px;
}

.dir-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.dir-input {
  flex: 1;
  padding: 6px 10px;
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  color: var(--t);
  background: rgba(var(--mg-rgb), 0.5);
  border: 1px solid rgba(var(--mg-rgb), 0.6);
  cursor: pointer;
}

.dir-input-base {
  flex: 2;
}

.dir-input-sub {
  flex: 1;
}

.dir-input::placeholder {
  color: var(--t3);
}

.dir-sep {
  color: var(--t3);
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  flex-shrink: 0;
}

.btn-browse {
  font-family: 'Oswald', sans-serif;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 6px 12px;
  background: rgba(var(--mg-rgb), 0.5);
  border: 1px solid rgba(var(--mg-rgb), 0.6);
  color: var(--t2);
  cursor: pointer;
}

.btn-browse:hover {
  background: rgba(var(--mg-rgb), 0.8);
}

.install-confirm-row {
  margin-top: 8px;
}

.btn-install-confirm {
  margin-right: 10px;
  padding: 6px 16px;
}

.install-path {
  font-family: 'Rajdhani', sans-serif;
  font-size: 15px;
  font-weight: 600;
  color: var(--t);
}

.missing-buttons {
  display: flex;
  gap: 10px;
  margin: 12px 0;
}

.btn-gold,
.btn-blue {
  font-family: 'Oswald', sans-serif;
  font-size: 14px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 10px 24px;
  cursor: pointer;
  color: var(--ink);
  text-shadow: 0 1px 1px rgba(255, 255, 255, 0.2);
}

.btn-gold {
  background: linear-gradient(180deg, var(--gold-bright) 0%, var(--gold-dark) 100%);
  border: 1px solid var(--gold-bright);
  box-shadow: 0 4px 16px rgba(var(--gold-rgb), 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.25);
}

.btn-gold:hover {
  background: linear-gradient(180deg, var(--gold-bright) 0%, var(--gold) 100%);
  box-shadow: 0 6px 24px rgba(var(--gold-rgb), 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.btn-blue {
  background: linear-gradient(180deg, var(--blue-bright) 0%, var(--blue-dark) 100%);
  border: 1px solid var(--blue-bright);
  box-shadow: 0 4px 16px rgba(var(--blue-rgb), 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.25);
}

.btn-blue:hover {
  background: linear-gradient(180deg, var(--blue-bright) 0%, var(--blue) 100%);
  box-shadow: 0 6px 24px rgba(var(--blue-rgb), 0.45), inset 0 1px 0 rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.registry-error {
  display: block;
  margin-top: 10px;
  font-size: 13px;
  color: var(--dl-light);
}
</style>
