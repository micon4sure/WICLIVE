<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGameState } from '../composables/useGameState'

const live = ref(false)
const competitive = ref(false)
const loading = ref(true)
const error = ref('')
const {
  skipLauncher,
  skipLauncherAvailable,
  skipLauncherBusy,
  skipLauncherError,
  setSkipLauncher,
} = useGameState()

async function toggleSkipLauncher() {
  if (!skipLauncherAvailable.value || skipLauncherBusy.value) return
  await setSkipLauncher(!skipLauncher.value)
}

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

// User settings (wicgate.txt)
interface WicgateSettings {
  nointro: boolean
  playonline: boolean
  camera_fix: boolean
  hilite_own_color: string
  ignore_alt_tab: boolean
  no_cursor_speed: boolean
  nuke_warning: boolean
}

const userSettings = ref<WicgateSettings>({
  nointro: false,
  playonline: false,
  camera_fix: true,
  hilite_own_color: '',
  ignore_alt_tab: false,
  no_cursor_speed: false,
  nuke_warning: true,
})
const userError = ref('')
const launchError = ref('')
const settingsLoading = ref(true)

type BoolKey = 'camera_fix' | 'ignore_alt_tab' | 'no_cursor_speed' | 'nuke_warning'
type LaunchBoolKey = 'nointro' | 'playonline'

const boolSettings: { key: BoolKey; name: string; desc: string }[] = [
  { key: 'camera_fix', name: 'Camera Fix', desc: 'No fly-to on drop zone change' },
  { key: 'ignore_alt_tab', name: 'Ignore Alt-Tab', desc: 'Prevent idle when alt-tabbed' },
  { key: 'no_cursor_speed', name: 'No Cursor Speed', desc: 'Disable cursor acceleration' },
  { key: 'nuke_warning', name: 'Nuke Warning', desc: 'HUD alert on enemy nuke' },
]

const colorPresets = [
  { name: 'amber', hex: '#CFB408' },
  { name: 'azure', hex: '#57B9FF' },
  { name: 'coral', hex: '#FF7F50' },
  { name: 'cyan', hex: '#00FFFF' },
  { name: 'gold', hex: '#FFD700' },
  { name: 'lime', hex: '#88FF00' },
  { name: 'magenta', hex: '#FF00FF' },
  { name: 'orange', hex: '#FF8800' },
  { name: 'pink', hex: '#FF69B4' },
  { name: 'silver', hex: '#C0C0C0' },
  { name: 'white', hex: '#FFFFFF' },
  { name: 'yellow', hex: '#FFFF00' },
]

const highlightLabel = computed(() => {
  const val = userSettings.value.hilite_own_color
  if (!val) return 'Disabled'
  const preset = colorPresets.find(c => c.name === val)
  if (preset) return preset.name.charAt(0).toUpperCase() + preset.name.slice(1)
  return '#' + val
})

const currentColorHex = computed(() => {
  const val = userSettings.value.hilite_own_color
  if (!val) return '#FF8800'
  const preset = colorPresets.find(c => c.name === val)
  if (preset) return preset.hex
  return '#' + val
})

async function loadUserSettings() {
  settingsLoading.value = true
  userError.value = ''
  launchError.value = ''
  try {
    userSettings.value = await invoke<WicgateSettings>('get_wicgate_settings')
  } catch (e) {
    userError.value = String(e)
    launchError.value = String(e)
  } finally {
    settingsLoading.value = false
  }
}

async function toggleLaunchSetting(key: LaunchBoolKey) {
  if (settingsLoading.value) return

  launchError.value = ''
  const target = !userSettings.value[key]
  try {
    await invoke('set_wicgate_setting', { key, value: target ? '1' : '0' })
    userSettings.value[key] = target
  } catch (e) {
    launchError.value = String(e)
  }
}

async function toggleUserSetting(key: BoolKey) {
  userError.value = ''
  const target = !userSettings.value[key]
  try {
    await invoke('set_wicgate_setting', { key, value: target ? '1' : '0' })
    userSettings.value[key] = target
  } catch (e) {
    userError.value = String(e)
  }
}

async function setHighlightColor(color: string) {
  userError.value = ''
  try {
    await invoke('set_wicgate_setting', { key: 'hilite_own_color', value: color })
    userSettings.value.hilite_own_color = color
  } catch (e) {
    userError.value = String(e)
  }
}

function toggleHighlight() {
  setHighlightColor(userSettings.value.hilite_own_color ? '' : 'orange')
}

function handleColorWheel(event: Event) {
  const hex = (event.target as HTMLInputElement).value.replace('#', '').toUpperCase()
  if (/^[0-9A-F]{6}$/.test(hex)) {
    setHighlightColor(hex)
  }
}

onMounted(async () => {
  loadState()
  loadUserSettings()
})
</script>

<template>
  <div class="config-section">
    <!-- Executable Config -->
    <div class="config-header">
      <h3>Game Startup</h3>
      <span class="config-sub">Launch options — changes apply on next launch</span>
    </div>

    <div
      class="config-card startup-card"
      :class="{ active: skipLauncher || userSettings.nointro || userSettings.playonline }"
    >
      <div
        class="card-top"
        :class="{ disabled: !skipLauncherAvailable || skipLauncherBusy }"
        role="switch"
        :aria-checked="skipLauncher"
        :aria-disabled="!skipLauncherAvailable || skipLauncherBusy"
        :tabindex="skipLauncherAvailable && !skipLauncherBusy ? 0 : -1"
        @click="toggleSkipLauncher"
        @keydown.enter.prevent="toggleSkipLauncher"
        @keydown.space.prevent="toggleSkipLauncher"
      >
        <div class="toggle-track" :class="{ on: skipLauncher }">
          <div class="toggle-thumb" />
        </div>
        <div class="card-title-area">
          <span class="card-title">Skip Welcome Launcher</span>
          <span class="card-desc">
            <template v-if="skipLauncherBusy">Checking installed executable...</template>
            <template v-else-if="!skipLauncherAvailable">Unavailable for this game executable</template>
            <template v-else-if="skipLauncher">Start World in Conflict directly</template>
            <template v-else>Show the Start Game window</template>
          </span>
        </div>
      </div>
      <div class="card-detail">
        <div
          class="setting-row"
          :class="{ disabled: settingsLoading }"
          role="switch"
          :aria-checked="userSettings.nointro"
          :aria-disabled="settingsLoading"
          :tabindex="settingsLoading ? -1 : 0"
          @click="toggleLaunchSetting('nointro')"
          @keydown.enter.prevent="toggleLaunchSetting('nointro')"
          @keydown.space.prevent="toggleLaunchSetting('nointro')"
        >
          <div class="toggle-track" :class="{ on: userSettings.nointro }">
            <div class="toggle-thumb" />
          </div>
          <div class="setting-text">
            <span class="setting-name">Skip Intro Videos</span>
            <span class="setting-desc">Skip publisher logos and intro movie (-nointro)</span>
          </div>
        </div>
        <div
          class="setting-row"
          :class="{ disabled: settingsLoading }"
          role="switch"
          :aria-checked="userSettings.playonline"
          :aria-disabled="settingsLoading"
          :tabindex="settingsLoading ? -1 : 0"
          @click="toggleLaunchSetting('playonline')"
          @keydown.enter.prevent="toggleLaunchSetting('playonline')"
          @keydown.space.prevent="toggleLaunchSetting('playonline')"
        >
          <div class="toggle-track" :class="{ on: userSettings.playonline }">
            <div class="toggle-thumb" />
          </div>
          <div class="setting-text">
            <span class="setting-name">Straight to Multiplayer</span>
            <span class="setting-desc">Open the multiplayer login screen (-playonline)</span>
          </div>
        </div>
      </div>
      <div v-if="skipLauncherError" class="config-error startup-error">{{ skipLauncherError }}</div>
      <div v-if="launchError" class="config-error startup-error">{{ launchError }}</div>
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

      <!-- User Settings -->
      <div class="config-card" :class="{ active: userSettings.camera_fix || !!userSettings.hilite_own_color || userSettings.ignore_alt_tab || userSettings.no_cursor_speed || userSettings.nuke_warning }">
        <div class="user-settings-header">
          <span class="card-title">User Settings</span>
          <span class="card-desc">wicgate.txt — requires restart</span>
        </div>
        <div v-if="userError" class="config-error">{{ userError }}</div>
        <div class="card-detail">
          <!-- Highlight color (top) -->
          <div class="setting-row" @click="toggleHighlight">
            <div class="toggle-track" :class="{ on: !!userSettings.hilite_own_color }">
              <div class="toggle-thumb" />
            </div>
            <div class="setting-text">
              <span class="setting-name">Unit Highlight</span>
              <span class="setting-desc">{{ highlightLabel }}</span>
            </div>
          </div>
          <div v-if="userSettings.hilite_own_color" class="color-picker">
            <div class="color-grid">
              <button
                v-for="c in colorPresets"
                :key="c.name"
                class="color-swatch"
                :class="{ active: userSettings.hilite_own_color === c.name }"
                :style="{ backgroundColor: c.hex }"
                :title="c.name.charAt(0).toUpperCase() + c.name.slice(1)"
                @click="setHighlightColor(c.name)"
              />
            </div>
            <div class="color-custom-row">
              <input
                type="color"
                :value="currentColorHex"
                class="color-input-native"
                title="Custom color"
                @input="handleColorWheel"
              />
              <span class="color-hex-label">{{ currentColorHex.toUpperCase() }}</span>
            </div>
          </div>
          <div v-if="userSettings.hilite_own_color" class="detail-sep" />
          <!-- Boolean toggles -->
          <div v-for="s in boolSettings" :key="s.key" class="setting-row" @click="toggleUserSetting(s.key)">
            <div class="toggle-track" :class="{ on: userSettings[s.key] }">
              <div class="toggle-thumb" />
            </div>
            <div class="setting-text">
              <span class="setting-name">{{ s.name }}</span>
              <span class="setting-desc">{{ s.desc }}</span>
            </div>
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
  font-size: 20px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-primary);
}
.config-sub {
  font-size: 15px;
  font-weight: 500;
  line-height: 1.35;
  color: var(--text-secondary);
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
  grid-template-columns: 1fr 1fr 1fr;
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
.card-top.disabled {
  cursor: not-allowed;
  opacity: 0.65;
}
.card-top.disabled:hover { background: transparent; }

.startup-error {
  border-width: 1px 0 0;
}

.card-title-area {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.card-title {
  font-family: 'Oswald', sans-serif;
  font-size: 17px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-primary);
}
.card-desc {
  font-size: 15px;
  font-weight: 500;
  line-height: 1.35;
  color: var(--text-secondary);
}

.card-detail {
  padding: 10px 16px 14px;
  border-top: 1px solid rgba(var(--mid-gray-rgb), 0.3);
}

.detail-grid {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 5px 12px;
  font-size: 15px;
  font-weight: 500;
  line-height: 1.3;
}

.detail-sep {
  height: 1px;
  background: rgba(var(--mid-gray-rgb), 0.3);
  margin: 8px 0;
}

.bind-key {
  font-family: 'Rajdhani', sans-serif;
  font-weight: 700;
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
/* User settings */
.user-settings-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 14px 16px;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  cursor: pointer;
}
.setting-row:hover {
  background: rgba(var(--mid-gray-rgb), 0.15);
}
.setting-row.disabled {
  cursor: not-allowed;
  opacity: 0.65;
}
.setting-row.disabled:hover {
  background: transparent;
}

.setting-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.setting-name {
  font-family: 'Oswald', sans-serif;
  font-size: 15px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 13px;
  font-weight: 500;
  line-height: 1.35;
  color: var(--text-secondary);
}

/* Color picker */
.color-picker {
  padding: 6px 0 2px;
}

.color-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 6px;
}

.color-swatch {
  aspect-ratio: 1;
  width: 100%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color 0.2s ease;
}
.color-swatch:hover {
  border-color: rgba(var(--mid-gray-muted-rgb), 0.8);
}
.color-swatch.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 4px rgba(var(--text-primary-rgb), 0.4);
}

.color-custom-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

.color-input-native {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border-default);
  border-radius: 4px;
  cursor: pointer;
  background: transparent;
  padding: 0;
}
.color-input-native::-webkit-color-swatch-wrapper {
  padding: 2px;
}
.color-input-native::-webkit-color-swatch {
  border: none;
  border-radius: 2px;
}

.color-hex-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  font-family: monospace;
}
</style>
