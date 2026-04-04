<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const live = ref(false)
const competitive = ref(false)
const loading = ref(true)
const error = ref('')

// Legacy proxy state (cl_hook = "current" environment)
const legacyInstalled = ref(false)
const legacyActive = ref(false)
const legacyDownloading = ref(false)
const legacyProgress = ref(0)
const legacyError = ref('')

async function loadState() {
  loading.value = true
  error.value = ''
  try {
    const [l, c] = await invoke<[boolean, boolean]>('get_autoexec_state')
    live.value = l
    competitive.value = c
  } catch (e) {
    error.value = String(e)
  }
  loading.value = false
}

async function loadLegacyState() {
  try {
    legacyInstalled.value = await invoke<boolean>('check_legacy_proxy')
    if (legacyInstalled.value) {
      legacyActive.value = await invoke<boolean>('is_legacy_proxy_active')
    }
  } catch (_) {}
}

async function switchToCurrent() {
  legacyError.value = ''
  if (!legacyInstalled.value) {
    legacyDownloading.value = true
    legacyProgress.value = 0
    try {
      await invoke('download_legacy_proxy')
      await invoke('install_legacy_proxy')
      legacyInstalled.value = true
    } catch (e) {
      legacyError.value = String(e)
      legacyDownloading.value = false
      return
    }
    legacyDownloading.value = false
  }
  try {
    await invoke('activate_legacy_proxy')
    legacyActive.value = true
  } catch (e) {
    legacyError.value = String(e)
  }
}

async function switchToPbe() {
  if (!legacyActive.value) return
  legacyError.value = ''
  try {
    await invoke('deactivate_legacy_proxy')
    legacyActive.value = false
  } catch (e) {
    legacyError.value = String(e)
  }
}

async function toggleLive() {
  error.value = ''
  const target = !live.value
  try {
    await invoke('set_live_settings', { enabled: target })
    live.value = target
  } catch (e) {
    error.value = String(e)
  }
}

async function toggleCompetitive() {
  error.value = ''
  const target = !competitive.value
  try {
    await invoke('set_competitive_settings', { enabled: target })
    competitive.value = target
  } catch (e) {
    error.value = String(e)
  }
}

onMounted(async () => {
  loadState()
  loadLegacyState()
  listen<{ stage: string; downloaded: number; total: number }>('legacy-proxy-progress', (e) => {
    if (e.payload.stage === 'downloading' && e.payload.total > 0) {
      legacyProgress.value = Math.round((e.payload.downloaded / e.payload.total) * 100)
    }
  })
})
</script>

<template>
  <div class="config-section">
    <!-- Server Toggle -->
    <div class="config-header">
      <h3>Server</h3>
      <span class="config-sub">Switch between <em>current</em> and <em>public beta environment</em> (PBE)</span>
    </div>

    <div v-if="legacyError" class="config-error">{{ legacyError }}</div>

    <div class="server-toggle">
      <button
        class="server-btn server-btn-live"
        :class="{ active: legacyActive, switching: legacyDownloading }"
        :disabled="legacyDownloading"
        @click="switchToCurrent"
      >
        <div class="server-btn-glow" />
        <div class="server-btn-content">
          <span class="server-label">CURRENT</span>
          <span class="server-desc">{{ legacyDownloading ? `Downloading ${legacyProgress}%` : 'Live environment' }}</span>
        </div>
      </button>
      <button
        class="server-btn server-btn-pbe"
        :class="{ active: !legacyActive, switching: legacyDownloading }"
        :disabled="legacyDownloading"
        @click="switchToPbe"
      >
        <div class="server-btn-glow" />
        <div class="server-btn-content">
          <span class="server-label">PBE</span>
          <span class="server-desc">Public beta environment</span>
        </div>
      </button>
    </div>

    <!-- Autoexec Config -->
    <div class="config-header">
      <h3>wicautoexec.txt</h3>
      <span class="config-sub">Game config presets — changes require game restart</span>
    </div>

    <div v-if="error" class="config-error">{{ error }}</div>

    <div class="config-columns">
      <!-- Live Keybinds -->
      <div class="config-card" :class="{ active: live }">
        <div class="card-top" @click="toggleLive">
          <div class="toggle-track" :class="{ on: live }">
            <div class="toggle-thumb" />
          </div>
          <div class="card-title-area">
            <span class="card-title">Live Settings</span>
            <span class="card-desc">TA hotkeys and camera freedom</span>
          </div>
        </div>
        <div class="card-detail">
          <div class="detail-grid">
            <span class="bind-key">F1</span><span class="bind-val">Aerial Recon</span>
            <span class="bind-key">F2</span><span class="bind-val">Air-to-Air</span>
            <span class="bind-key">F3</span><span class="bind-val">Tankbuster</span>
            <span class="bind-key">F4</span><span class="bind-val">Light Artillery</span>
            <span class="bind-key">F5</span><span class="bind-val">Heavy Artillery</span>
            <span class="bind-key">7</span><span class="bind-val">Jeep Drops</span>
            <span class="bind-key">8</span><span class="bind-val">Tank Drops</span>
            <span class="bind-key">9</span><span class="bind-val">Airbornes</span>
            <span class="bind-key">0</span><span class="bind-val">Cluster Bomb</span>
          </div>
          <div class="detail-sep" />
          <div class="detail-grid">
            <span class="bind-key">Camera</span><span class="bind-val">Free rotation</span>
            <span class="bind-key">Height</span><span class="bind-val">1500 max</span>
          </div>
        </div>
      </div>

      <!-- Competitive Settings -->
      <div class="config-card" :class="{ active: competitive }">
        <div class="card-top" @click="toggleCompetitive">
          <div class="toggle-track" :class="{ on: competitive }">
            <div class="toggle-thumb" />
          </div>
          <div class="card-title-area">
            <span class="card-title">Competitive Settings</span>
            <span class="card-desc">Visual clarity for competitive play</span>
          </div>
        </div>
        <div class="card-detail">
          <div class="detail-grid">
            <span class="bind-key">Fog</span><span class="bind-val">Disabled</span>
            <span class="bind-key">Clouds</span><span class="bind-val">Disabled</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Server toggle */
.server-toggle {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.server-btn {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px 16px;
  border: 1px solid rgba(var(--mid-gray-muted-rgb), 0.5);
  background: rgba(var(--mid-gray-dark-rgb), 0.6);
  cursor: pointer;
  overflow: hidden;
  transition: border-color 0.3s ease, background 0.3s ease, box-shadow 0.3s ease;
}

.server-btn:hover:not(:disabled):not(.active) {
  border-color: rgba(var(--mid-gray-muted-rgb), 0.8);
  background: rgba(var(--mid-gray-rgb), 0.4);
}

.server-btn:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.server-btn-glow {
  position: absolute;
  inset: 0;
  opacity: 0;
  transition: opacity 0.4s ease;
  pointer-events: none;
}

.server-btn.active .server-btn-glow {
  opacity: 1;
}

.server-btn-live .server-btn-glow {
  background: radial-gradient(ellipse at 50% 120%, rgba(var(--blue-rgb), 0.2) 0%, transparent 70%);
}
.server-btn-pbe .server-btn-glow {
  background: radial-gradient(ellipse at 50% 120%, rgba(var(--c-cta-rgb), 0.2) 0%, transparent 70%);
}

.server-btn-live.active {
  border-color: rgba(var(--blue-rgb), 0.5);
  background: rgba(var(--mid-gray-dark-rgb), 0.8);
  box-shadow:
    0 0 20px rgba(var(--blue-rgb), 0.15),
    inset 0 -2px 0 rgba(var(--blue-rgb), 0.6);
}

.server-btn-pbe.active {
  border-color: rgba(var(--c-cta-rgb), 0.5);
  background: rgba(var(--mid-gray-dark-rgb), 0.8);
  box-shadow:
    0 0 20px rgba(var(--c-cta-rgb), 0.15),
    inset 0 -2px 0 rgba(var(--c-cta-rgb), 0.6);
}

.server-btn-content {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  z-index: 1;
}

.server-label {
  font-family: 'Oswald', sans-serif;
  font-size: 22px;
  font-weight: 700;
  letter-spacing: 3px;
  text-transform: uppercase;
  color: var(--text-tertiary);
  transition: color 0.3s ease;
}

.server-btn.active .server-label {
  color: var(--text-primary);
}

.server-btn-live.active .server-label {
  color: var(--blue);
}

.server-btn-pbe.active .server-label {
  color: var(--silver);
}

.server-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  letter-spacing: 0.5px;
  transition: color 0.3s ease;
}

.server-btn.active .server-desc {
  color: var(--text-secondary);
}

.server-btn.switching .server-desc {
  color: var(--text-tertiary);
}

.config-header h3 {
  margin: 0 0 4px;
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-primary);
}
.config-sub {
  font-size: 14px;
  color: var(--text-tertiary);
}

.config-error {
  padding: 10px 16px;
  background: rgba(var(--c-error-rgb), 0.15);
  border: 1px solid rgba(var(--c-error-rgb), 0.3);
  color: var(--c-error);
  font-size: 13px;
}

.config-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.config-card {
  border: 1px solid var(--border-default);
  background: rgba(var(--bg-rgb), 0.85);
  transition: border-color 0.3s ease;
}
.config-card.active {
  border-color: rgba(var(--c-brand-rgb), 0.4);
}

.card-top {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
}
.card-top:hover { background: rgba(var(--mid-gray-rgb), 0.2); }

.card-title-area {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.card-title {
  font-family: 'Oswald', sans-serif;
  font-size: 15px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-primary);
}
.card-desc {
  font-size: 13px;
  color: var(--text-tertiary);
}

.card-detail {
  padding: 10px 16px 14px;
  border-top: 1px solid rgba(var(--mid-gray-rgb), 0.3);
}

.detail-grid {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 3px 10px;
  font-size: 13px;
}

.detail-sep {
  height: 1px;
  background: rgba(var(--mid-gray-rgb), 0.3);
  margin: 8px 0;
}

.bind-key {
  font-family: 'Rajdhani', sans-serif;
  font-weight: 600;
  color: var(--c-brand);
  text-align: right;
}
.bind-val {
  color: var(--text-secondary);
}

/* Toggle */
.toggle-track {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: rgba(var(--mid-gray-muted-rgb), 0.6);
  border: 1px solid rgba(var(--mid-gray-muted-rgb), 0.8);
  position: relative;
  transition: background 0.25s ease, border-color 0.25s ease;
  flex-shrink: 0;
}
.toggle-track.on {
  background: rgba(var(--c-brand-rgb), 0.85);
  border-color: rgba(var(--c-brand-rgb), 0.95);
}
.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--text-secondary);
  transition: transform 0.25s ease, background 0.25s ease;
}
.toggle-track.on .toggle-thumb {
  transform: translateX(18px);
  background: #fff;
}
</style>
