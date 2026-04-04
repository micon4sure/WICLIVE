<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { useGameState } from '../composables/useGameState'

const { readinessActions, checking, installed, initialized, isReady, check } = useGameState()

const appWindow = getCurrentWindow()
const version = ref('')

const labels: Record<string, string> = {
  vcredist: 'VC++',
  patch: 'Patches',
  laa: 'LAA',
  cdkey: 'CD Key',
  proxy_installed: 'Proxy',
  proxy_current: 'Proxy Ver.',
}

const pills = computed(() =>
  Object.entries(readinessActions.value).map(([id, a]) => {
    let status: string
    if (checking.value && !a.need) status = 'checking'
    else if (a.has) status = 'done'
    else if (a.need) status = 'needed'
    else status = 'checking'
    return { id, label: labels[id] || id, status, detail: a.detail }
  })
)

onMounted(async () => {
  version.value = await getVersion()
})

function minimize() { appWindow.minimize() }
function toggleMaximize() { appWindow.toggleMaximize() }
function close() { appWindow.close() }

async function launchGame() {
  await invoke('start_game')
}

const isDev = import.meta.env.DEV

async function undoPatches() {
  await invoke('reset_game', { variant: 'wic.1.0.0.nolaa.exe' })
  check()
}

async function undoLaa() {
  await invoke('unset_laa_flag')
  check()
}

async function undoProxy() {
  await invoke('remove_proxy')
  check()
}

async function removeCdKey() {
  await invoke('set_cd_key', { key: '' })
  check()
}

const debugCopied = ref(false)
async function copyDebug() {
  const info = await invoke<string>('get_debug_info')
  await navigator.clipboard.writeText(info)
  debugCopied.value = true
  setTimeout(() => { debugCopied.value = false }, 2000)
}
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-brand" data-tauri-drag-region>
      <img src="../assets/icon.png" alt="" class="titlebar-icon" data-tauri-drag-region />
      <span class="titlebar-title" data-tauri-drag-region>WIC LIVE</span>
    </div>
    <div class="titlebar-controls">
      <button class="titlebar-btn" @click="minimize">&#x2013;</button>
      <button class="titlebar-btn" @click="toggleMaximize">&#x25A1;</button>
      <button class="titlebar-btn titlebar-close" @click="close">&#x2715;</button>
    </div>
  </div>
  <div class="header" :class="{ 'header-copied': debugCopied }" @click="copyDebug">
    <img src="../assets/wiclive.png" alt="WIC LIVE" class="header-logo" />
    <small class="header-version">{{ version }}</small>

    <div v-if="installed && initialized" class="status-area">
      <div v-if="isDev" class="dev-toolbar">
        <button class="dev-btn" @click.stop="undoPatches">undo patches</button>
        <button class="dev-btn" @click.stop="undoLaa">undo laa</button>
        <button class="dev-btn" @click.stop="removeCdKey">remove cdkey</button>
        <button class="dev-btn" @click.stop="undoProxy">undo proxy</button>
      </div>
      <div class="status-bar">
        <div v-for="pill in pills" :key="pill.id" class="status-pill" :class="pill.status">
          <span class="pill-icon">
            <template v-if="pill.status === 'done'">&#x2713;</template>
            <template v-else-if="pill.status === 'needed'">&#x25CB;</template>
            <template v-else>&middot;&middot;</template>
          </span>
          <span class="pill-label">{{ pill.label }}</span>
          <span class="pill-detail" :class="{ 'pill-detail-hidden': !(pill.id === 'patch' || pill.id === 'cdkey' || pill.id === 'proxy_installed' || pill.id === 'proxy_current') }">{{ pill.detail || '&nbsp;' }}</span>
        </div>
      </div>
    </div>

    <button v-if="installed && initialized" class="btn btn-launch header-launch" :disabled="!isReady" @click.stop="launchGame">Start Game</button>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  height: 32px;
  background: linear-gradient(180deg, var(--graphite-dark) 0%, var(--graphite) 100%);
  border-bottom: 1px solid rgba(var(--mid-gray-rgb), 0.25);
  flex-shrink: 0;
}

.header {
  display: flex;
  align-items: stretch;
  padding: 0 25px;
  background: rgba(0, 0, 0, 0.75);
  border-bottom: 2px solid rgba(var(--c-accent-rgb), 0.45);
  flex-shrink: 0;
}

.header-logo {
  height: 69px;
  width: auto;
  align-self: center;
  margin: 20px 0;
}

.header-version {
  margin-left: 10px;
  font-size: 12px;
  color: #fff;
  align-self: flex-end;
  padding-bottom: 20px;
}

.status-area {
  display: flex;
  flex-direction: column;
  align-self: stretch;
  margin-left: auto;
  margin-right: 20px;
}

.dev-toolbar {
  display: flex;
  gap: 1px;
  justify-content: flex-end;
  padding: 4px 0 0;
}

.dev-btn {
  font-family: 'Rajdhani', sans-serif;
  font-size: 10px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 2px 8px;
  background: rgba(var(--c-accent-rgb), 0.3);
  border: 1px solid rgba(var(--c-accent-rgb), 0.4);
  color: var(--c-accent);
  cursor: pointer;
  transition: var(--transition);
}

.dev-btn:hover {
  background: rgba(var(--c-accent-rgb), 0.6);
  color: #fff;
}

.status-bar {
  display: flex;
  align-items: stretch;
  flex: 1;
  gap: 0;
}

.status-pill {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 15px 14px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  white-space: nowrap;
  color: var(--text-tertiary);
  border-right: none;
  position: relative;
}

.status-pill::after {
  content: '';
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 1px;
  background: linear-gradient(to bottom, transparent, rgba(255, 255, 255, 0.15) 30%, rgba(255, 255, 255, 0.15) 70%, transparent);
}

.status-pill:last-child::after {
  display: none;
}

.status-pill:first-child::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 1px;
  background: linear-gradient(to bottom, transparent, rgba(255, 255, 255, 0.15) 30%, rgba(255, 255, 255, 0.15) 70%, transparent);
}

.pill-icon {
  font-size: 10px;
  font-weight: 700;
}

.pill-label {
  font-family: 'Oswald', sans-serif;
  font-weight: 500;
  letter-spacing: 1px;
}

.pill-detail {
  font-family: 'Rajdhani', sans-serif;
  font-size: 11px;
  font-weight: 500;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  text-transform: none;
  line-height: 1.2;
}

.pill-detail-hidden {
  visibility: hidden;
}

.status-pill.done,
.status-pill.fixed {
  color: var(--c-success);
}

.status-pill.needed {
  color: var(--c-pending);
}

.status-pill.applying {
  color: var(--c-progress);
}

.status-pill.error {
  color: var(--c-error);
}

.status-pill.checking {
  color: var(--text-tertiary);
  animation: pulse 2s ease-in-out infinite;
}

.header-launch {
  align-self: center;
}

.header:hover {
  background: linear-gradient(180deg, rgba(var(--graphite-light-rgb), 1) 0%, rgba(var(--graphite-dark-rgb), 1) 100%);
  cursor: pointer;
}

.header {
  position: relative;
}

.header-copied::after {
  content: '';
  position: absolute;
  inset: 0;
  background: rgba(var(--blue-rgb), 0.15);
  pointer-events: none;
  animation: header-pulse 0.8s ease-in-out forwards;
}

@keyframes header-pulse {
  0% { opacity: 0; }
  40% { opacity: 1; }
  100% { opacity: 0; }
}

.titlebar-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 12px;
  margin-right: auto;
}

.titlebar-icon {
  height: 18px;
  width: auto;
  opacity: 0.7;
}

.titlebar-title {
  font-family: 'Oswald', sans-serif;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  color: var(--text-tertiary);
}

.titlebar-controls {
  display: flex;
  height: 100%;
}

.titlebar-btn {
  width: 46px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.titlebar-btn:hover {
  background: rgba(var(--mid-gray-rgb), 0.4);
  color: var(--text-primary);
}

.titlebar-close:hover {
  background: rgba(var(--c-error-rgb), 0.85);
  color: #fff;
}
</style>
