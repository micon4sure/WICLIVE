<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

type ActionStatus = 'checking' | 'done' | 'needed' | 'error'

interface Action {
  id: string
  label: string
  status: ActionStatus
  detail?: string
}

type AppMode = 'checking' | 'not-installed' | 'broken' | 'needs-fixes' | 'ready'

const appWindow = getCurrentWindow()
const version = ref('')
const installState = ref<'ok' | 'broken' | 'not-installed'>('ok')

const actions = ref<Action[]>([
  { id: 'vcredist', label: 'VC++', status: 'checking' },
  { id: 'patch', label: 'Patches', status: 'checking' },
  { id: 'laa', label: 'LAA', status: 'checking' },
  { id: 'cdkey', label: 'CD Key', status: 'checking' },
  { id: 'hooks', label: 'Proxy', status: 'checking' },
])

const mode = computed<AppMode>(() => {
  if (installState.value === 'not-installed') return 'not-installed'
  if (installState.value === 'broken') return 'broken'
  const statuses = actions.value.map(a => a.status)
  if (statuses.some(s => s === 'checking')) return 'checking'
  if (statuses.every(s => s === 'done' || s === 'fixed')) return 'ready'
  return 'needs-fixes'
})

defineExpose({ actions, mode, runChecks })

function find(id: string) {
  return actions.value.find(a => a.id === id)!
}

async function runChecks() {
  for (const a of actions.value) {
    a.status = 'checking'
    a.detail = undefined
  }

  const installPath = await invoke<string | null>('get_install_path')
  if (!installPath) {
    const hasRegistry = await invoke<boolean>('has_registry_install_path')
    installState.value = hasRegistry ? 'broken' : 'not-installed'
    for (const a of actions.value) a.status = 'error'
    return
  }
  installState.value = 'ok'

  try {
    const ok = await invoke<boolean>('check_vcredist')
    const a = find('vcredist')
    a.status = ok ? 'done' : 'needed'
    a.detail = ok ? 'Installed' : 'Missing'
  } catch { find('vcredist').status = 'error' }

  try {
    const ver = await invoke<{ major: number; minor: number; patch: number; build: number }>('get_game_version')
    const a = find('patch')
    const str = `${ver.major}.${ver.minor}.${ver.patch}.${ver.build}`
    const ok = ver.patch === 1 && ver.build === 1
    a.status = ok ? 'done' : 'needed'
    a.detail = ok ? str : `${str} → 1.0.1.1`
  } catch { find('patch').status = 'error' }

  try {
    const laa = await invoke<boolean>('get_laa_flag')
    const a = find('laa')
    a.status = laa ? 'done' : 'needed'
    a.detail = laa ? 'Enabled' : 'Not set'
  } catch { find('laa').status = 'error' }

  try {
    const key = await invoke<string>('get_cd_key')
    const a = find('cdkey')
    const ok = key.length > 0
    a.status = ok ? 'done' : 'needed'
    a.detail = ok ? key : 'Not set'
  } catch { find('cdkey').status = 'error' }

  try {
    const ok = await invoke<boolean>('check_hooks')
    const a = find('hooks')
    if (ok) {
      const ver = await invoke<string>('get_hooks_version')
      const latest = await invoke<string>('get_latest_proxy_version').catch(() => '')
      if (latest && ver.trim() !== latest.trim()) {
        a.status = 'needed'
        a.detail = `${ver.trim()} → ${latest.trim()}`
      } else {
        a.status = 'done'
        a.detail = ver.trim()
      }
    } else {
      a.status = 'needed'
      a.detail = 'Not installed'
    }
  } catch { find('hooks').status = 'error' }

}

onMounted(async () => {
  version.value = await getVersion()
  runChecks()
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
  runChecks()
}

async function undoLaa() {
  await invoke('unset_laa_flag')
  runChecks()
}

async function undoProxy() {
  await invoke('remove_proxy')
  runChecks()
}

async function removeCdKey() {
  await invoke('set_cd_key', { key: '' })
  runChecks()
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

    <div v-if="mode !== 'broken' && mode !== 'not-installed'" class="status-area">
      <div v-if="isDev" class="dev-toolbar">
        <button class="dev-btn" @click.stop="undoPatches">undo patches</button>
        <button class="dev-btn" @click.stop="undoLaa">undo laa</button>
        <button class="dev-btn" @click.stop="removeCdKey">remove cdkey</button>
        <button class="dev-btn" @click.stop="undoProxy">undo proxy</button>
      </div>
      <div class="status-bar">
        <div v-for="action in actions" :key="action.id" class="status-pill" :class="action.status">
          <span class="pill-icon">
            <template v-if="action.status === 'done' || action.status === 'fixed'">&#x2713;</template>
            <template v-else-if="action.status === 'needed'">&#x25CB;</template>
            <template v-else-if="action.status === 'error'">&#x2717;</template>
            <template v-else>&middot;&middot;</template>
          </span>
          <span class="pill-label">{{ action.label }}</span>
          <span class="pill-detail" :class="{ 'pill-detail-hidden': !(action.id === 'patch' || action.id === 'cdkey' || action.id === 'hooks') }">{{ action.detail || '&nbsp;' }}</span>
        </div>
      </div>
    </div>

    <button v-if="mode !== 'broken' && mode !== 'not-installed'" class="btn btn-launch header-launch" :disabled="mode !== 'ready'" @click.stop="launchGame">Start Game</button>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  height: 32px;
  background: linear-gradient(180deg, var(--graphite-dark) 0%, var(--graphite) 100%);
  border-bottom: 1px solid rgba(var(--mg-rgb), 0.25);
  flex-shrink: 0;
}

.header {
  display: flex;
  align-items: stretch;
  padding: 0 25px;
  background: rgba(0, 0, 0, 0.75);
  border-bottom: 2px solid rgba(var(--dl-rgb), 0.45);
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
  background: rgba(var(--dl-rgb), 0.3);
  border: 1px solid rgba(var(--dl-rgb), 0.4);
  color: var(--dl-light);
  cursor: pointer;
  transition: var(--tr);
}

.dev-btn:hover {
  background: rgba(var(--dl-rgb), 0.6);
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
  color: var(--t3);
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
  color: var(--t2);
  text-transform: none;
  line-height: 1.2;
}

.pill-detail-hidden {
  visibility: hidden;
}

.status-pill.done,
.status-pill.fixed {
  color: var(--g);
}

.status-pill.needed {
  color: var(--sw);
}

.status-pill.applying {
  color: var(--b);
}

.status-pill.error {
  color: var(--dl-light);
}

.status-pill.checking {
  color: var(--t3);
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
  background: rgba(var(--b-rgb), 0.15);
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
  color: var(--t3);
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
  color: var(--t3);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.titlebar-btn:hover {
  background: rgba(var(--mg-rgb), 0.4);
  color: var(--t);
}

.titlebar-close:hover {
  background: rgba(var(--dl-light-rgb), 0.85);
  color: #fff;
}
</style>
