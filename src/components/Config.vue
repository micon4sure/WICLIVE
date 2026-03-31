<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const live = ref(false)
const competitive = ref(false)
const loading = ref(true)
const error = ref('')

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

onMounted(loadState)
</script>

<template>
  <div class="config-section">
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
            <span class="card-title">Live Keybinds</span>
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

.config-header h3 {
  margin: 0 0 4px;
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--t);
}
.config-sub {
  font-size: 14px;
  color: var(--t3);
}

.config-error {
  padding: 10px 16px;
  background: rgba(var(--dl-light-rgb), 0.15);
  border: 1px solid rgba(var(--dl-light-rgb), 0.3);
  color: var(--dl-light);
  font-size: 13px;
}

.config-columns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.config-card {
  border: 1px solid var(--bd);
  background: rgba(var(--bg-rgb), 0.85);
  transition: border-color 0.3s ease;
}
.config-card.active {
  border-color: rgba(var(--sw-rgb), 0.4);
}

.card-top {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  cursor: pointer;
  transition: background 0.2s ease;
}
.card-top:hover { background: rgba(var(--mg-rgb), 0.2); }

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
  color: var(--t);
}
.card-desc {
  font-size: 13px;
  color: var(--t3);
}

.card-detail {
  padding: 10px 16px 14px;
  border-top: 1px solid rgba(var(--mg-rgb), 0.3);
}

.detail-grid {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 3px 10px;
  font-size: 13px;
}

.detail-sep {
  height: 1px;
  background: rgba(var(--mg-rgb), 0.3);
  margin: 8px 0;
}

.bind-key {
  font-family: 'Rajdhani', sans-serif;
  font-weight: 600;
  color: var(--sw);
  text-align: right;
}
.bind-val {
  color: var(--t2);
}

/* Toggle */
.toggle-track {
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: rgba(var(--mg-muted-rgb), 0.6);
  border: 1px solid rgba(var(--mg-muted-rgb), 0.8);
  position: relative;
  transition: background 0.25s ease, border-color 0.25s ease;
  flex-shrink: 0;
}
.toggle-track.on {
  background: rgba(var(--sw-rgb), 0.85);
  border-color: rgba(var(--sw-rgb), 0.95);
}
.toggle-thumb {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--t2);
  transition: transform 0.25s ease, background 0.25s ease;
}
.toggle-track.on .toggle-thumb {
  transform: translateX(18px);
  background: #fff;
}
</style>
