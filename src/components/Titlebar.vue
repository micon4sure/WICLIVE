<script setup lang="ts">
import { ref, onMounted } from 'vue'
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

const appWindow = getCurrentWindow()
const version = ref('')
const allReady = ref(false)

const actions = ref<Action[]>([
  { id: 'laa', label: 'LAA', status: 'checking' },
  { id: 'vcredist', label: 'VC++', status: 'checking' },
  { id: 'patch', label: 'Patches', status: 'checking' },
  { id: 'cdkey', label: 'CD Key', status: 'checking' },
  { id: 'hooks', label: 'Hooks', status: 'checking' },
])

function find(id: string) {
  return actions.value.find(a => a.id === id)!
}

async function runChecks() {
  const installPath = await invoke<string | null>('get_install_path')
  if (!installPath) {
    for (const a of actions.value) a.status = 'error'
    return
  }

  try {
    const laa = await invoke<boolean>('get_laa_flag')
    const a = find('laa')
    a.status = laa ? 'done' : 'needed'
    a.detail = laa ? 'Enabled' : 'Not set'
  } catch { find('laa').status = 'error' }

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
      a.status = 'done'
      a.detail = ver.trim()
    } else {
      a.status = 'needed'
      a.detail = 'Not installed'
    }
  } catch { find('hooks').status = 'error' }

  allReady.value = actions.value.every(a => a.status === 'done')
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
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-controls">
      <button class="titlebar-btn" @click="minimize">&#x2013;</button>
      <button class="titlebar-btn" @click="toggleMaximize">&#x25A1;</button>
      <button class="titlebar-btn titlebar-close" @click="close">&#x2715;</button>
    </div>
  </div>
  <div class="header" data-tauri-drag-region>
    <img src="../assets/wiclive.png" alt="WIC LIVE" class="header-logo" data-tauri-drag-region />
    <small class="header-version" data-tauri-drag-region>{{ version }}</small>

    <div class="status-bar" data-tauri-drag-region>
      <div v-for="action in actions" :key="action.id" class="status-pill" :class="action.status">
        <span class="pill-icon">
          <template v-if="action.status === 'done'">&#x2713;</template>
          <template v-else-if="action.status === 'needed'">&#x25CB;</template>
          <template v-else-if="action.status === 'error'">&#x2717;</template>
          <template v-else>&middot;&middot;</template>
        </span>
        <span class="pill-label">{{ action.label }}</span>
        <span class="pill-detail" :class="{ 'pill-detail-hidden': !(action.id === 'patch' || action.id === 'cdkey' || action.id === 'hooks') }">{{ action.detail || '&nbsp;' }}</span>
      </div>
    </div>

    <button class="btn btn-launch header-launch" :disabled="!allReady" @click="launchGame">Start Game</button>
  </div>
</template>

<style scoped>
.titlebar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  height: 32px;
  background: var(--graphite-dark);
  flex-shrink: 0;
}

.header {
  display: flex;
  align-items: stretch;
  padding: 0 25px;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  border-bottom: 1px solid var(--bd);
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

.status-bar {
  display: flex;
  align-items: stretch;
  align-self: stretch;
  gap: 0;
  margin-left: auto;
  margin-right: 20px;
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

.status-pill.done {
  color: var(--g);
}

.status-pill.needed {
  color: var(--sw);
}

.status-pill.error {
  color: var(--dl-light);
}

.status-pill.checking {
  color: var(--t3);
}

.header-launch {
  align-self: center;
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
