import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ReadinessAction {
  need: boolean
  has: boolean
  detail: string
}

export interface InstallAction {
  has: boolean
  detail: string
}

const initialized = ref(false)
const installed = ref(false)
const broken = ref(false)
const checking = ref(false)
const wasFixed = ref(false)
const wasInstalled = ref(false)

const readinessActions = ref<Record<string, ReadinessAction>>({
  vcredist: { need: false, has: false, detail: '' },
  patch: { need: false, has: false, detail: '' },
  laa: { need: false, has: false, detail: '' },
  cdkey: { need: false, has: false, detail: '' },
  proxy_installed: { need: false, has: false, detail: '' },
  proxy_current: { need: false, has: false, detail: '' },
})

const installActions = ref<Record<string, InstallAction>>({
  download: { has: false, detail: '' },
  extract: { has: false, detail: '' },
  install: { has: false, detail: '' },
  cdkey: { has: false, detail: '' },
  shortcut: { has: false, detail: '' },
})

const needInstall = computed(() => !installed.value)
const needFix = computed(() => installed.value && Object.values(readinessActions.value).some(a => a.need && !a.has))
const isReady = computed(() => (!needFix.value || wasFixed.value) && (!needInstall.value || wasInstalled.value))

async function check() {
  checking.value = true

  const installPath = await invoke<string | null>('get_install_path')
  if (!installPath) {
    const hasRegistry = await invoke<boolean>('has_registry_install_path')
    installed.value = false
    broken.value = hasRegistry
    checking.value = false
    return
  }

  installed.value = true
  broken.value = false

  try {
    const has = await invoke<boolean>('check_vcredist')
    readinessActions.value.vcredist = { need: true, has, detail: has ? 'Installed' : 'Missing' }
  } catch {
    readinessActions.value.vcredist = { need: true, has: false, detail: 'Error' }
  }

  try {
    const ver = await invoke<{ major: number; minor: number; patch: number; build: number }>('get_game_version')
    const str = `${ver.major}.${ver.minor}.${ver.patch}.${ver.build}`
    const has = ver.patch === 1 && ver.build === 1
    readinessActions.value.patch = { need: true, has, detail: has ? str : `${str} → 1.0.1.1` }
  } catch {
    readinessActions.value.patch = { need: true, has: false, detail: 'Error' }
  }

  try {
    const has = await invoke<boolean>('get_laa_flag')
    readinessActions.value.laa = { need: true, has, detail: has ? 'Enabled' : 'Not set' }
  } catch {
    readinessActions.value.laa = { need: true, has: false, detail: 'Error' }
  }

  try {
    const key = await invoke<string>('get_cd_key')
    const has = key.length > 0 && key.toLowerCase() !== 'invalid'
    readinessActions.value.cdkey = { need: true, has, detail: has ? key : 'Not set' }
  } catch {
    readinessActions.value.cdkey = { need: true, has: false, detail: 'Error' }
  }

  try {
    const ok = await invoke<boolean>('check_proxy')
    if (ok) {
      readinessActions.value.proxy_installed = { need: true, has: true, detail: 'Installed' }
      const ver = await invoke<string>('get_proxy_version')
      const latest = await invoke<string>('get_latest_proxy_version').catch(() => '')
      const current = !latest || ver.trim() === latest.trim()
      readinessActions.value.proxy_current = { need: true, has: current, detail: current ? ver.trim() : `${ver.trim()} → ${latest.trim()}` }
    } else {
      readinessActions.value.proxy_installed = { need: true, has: false, detail: 'Not installed' }
      readinessActions.value.proxy_current = { need: false, has: false, detail: '' }
    }
  } catch {
    readinessActions.value.proxy_installed = { need: true, has: false, detail: 'Error' }
    readinessActions.value.proxy_current = { need: false, has: false, detail: '' }
  }

  checking.value = false
  initialized.value = true
}

async function onInstalled() {
  wasInstalled.value = true
  installed.value = true
  await check()
}

export function useGameState() {
  return {
    initialized,
    installed,
    broken,
    checking,
    wasFixed,
    wasInstalled,
    readinessActions,
    installActions,
    needInstall,
    needFix,
    isReady,
    check,
    onInstalled,
  }
}
check();