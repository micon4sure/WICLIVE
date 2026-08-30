import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ReadinessAction {
  need: boolean
  has: boolean
  detail: string
  warning?: boolean
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
const skipLauncher = ref(false)
const skipLauncherAvailable = ref(false)
const skipLauncherBusy = ref(true)
const skipLauncherError = ref('')
const compatibilityProxy = ref(false)
const proxySwitchBusy = ref(false)
const proxySwitchError = ref('')

const readinessActions = ref<Record<string, ReadinessAction>>({
  documents: { need: false, has: false, detail: '' },
  vcredist: { need: false, has: false, detail: '' },
  dx9: { need: false, has: false, detail: '' },
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
const needFix = computed(() => installed.value && Object.values(readinessActions.value).some(a => a.need && !a.has && !a.warning))
const hasWarnings = computed(() => installed.value && Object.values(readinessActions.value).some(a => a.need && !a.has && a.warning))
const isReady = computed(() => (!needFix.value || wasFixed.value) && (!needInstall.value || wasInstalled.value))

async function refreshSkipLauncher() {
  skipLauncherBusy.value = true
  skipLauncherError.value = ''
  try {
    skipLauncher.value = await invoke<boolean>('get_skip_launcher_flag')
    skipLauncherAvailable.value = true
  } catch (e) {
    skipLauncher.value = false
    skipLauncherAvailable.value = false
    skipLauncherError.value = String(e)
  } finally {
    skipLauncherBusy.value = false
  }
}

async function setSkipLauncher(enabled: boolean): Promise<boolean> {
  if (!skipLauncherAvailable.value || skipLauncherBusy.value) return false

  skipLauncherBusy.value = true
  skipLauncherError.value = ''
  try {
    skipLauncher.value = await invoke<boolean>('set_skip_launcher_flag', { enabled })
    skipLauncherAvailable.value = true
    return true
  } catch (e) {
    skipLauncherError.value = String(e)
    return false
  } finally {
    skipLauncherBusy.value = false
  }
}

async function check() {
  checking.value = true

  const installPath = await invoke<string | null>('get_install_path')
  if (!installPath) {
    const hasRegistry = await invoke<boolean>('has_registry_install_path')
    installed.value = false
    broken.value = hasRegistry
    skipLauncher.value = false
    skipLauncherAvailable.value = false
    skipLauncherBusy.value = false
    skipLauncherError.value = ''
    checking.value = false
    initialized.value = true
    return
  }

  installed.value = true
  broken.value = false

  try {
    await invoke<[boolean, boolean]>('get_autoexec_state')
    readinessActions.value.documents = { need: true, has: true, detail: 'Found' }
  } catch {
    readinessActions.value.documents = { need: true, has: false, detail: 'Not found', warning: true }
  }

  try {
    const has = await invoke<boolean>('check_vcredist')
    readinessActions.value.vcredist = { need: true, has, detail: has ? 'Installed' : 'Missing' }
  } catch {
    readinessActions.value.vcredist = { need: true, has: false, detail: 'Error' }
  }

  try {
    const has = await invoke<boolean>('check_dx9')
    readinessActions.value.dx9 = { need: true, has, detail: has ? 'Installed' : 'Missing' }
  } catch {
    readinessActions.value.dx9 = { need: true, has: false, detail: 'Error' }
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

  await refreshSkipLauncher()

  try {
    const key = await invoke<string>('get_cd_key')
    const has = key.length > 0 && key.toLowerCase() !== 'invalid'
    readinessActions.value.cdkey = { need: true, has, detail: has ? key : 'Not set' }
  } catch {
    readinessActions.value.cdkey = { need: true, has: false, detail: 'Error' }
  }

  try {
    compatibilityProxy.value = await invoke<boolean>('is_compatibility_proxy')
  } catch {
    compatibilityProxy.value = false
  }

  try {
    const compatibility = compatibilityProxy.value
    const ok = await invoke<boolean>('check_proxy', { compatibility })
    if (ok) {
      readinessActions.value.proxy_installed = { need: true, has: true, detail: 'Installed' }
      const ver = await invoke<string>('get_proxy_version', { compatibility })
      const latest = await invoke<string>('get_latest_proxy_version', { compatibility }).catch(() => '')
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

async function setCompatibilityProxy(enabled: boolean): Promise<boolean> {
  if (proxySwitchBusy.value || enabled === compatibilityProxy.value) return false

  proxySwitchBusy.value = true
  proxySwitchError.value = ''
  try {
    await invoke<string>('install_proxy', { compatibility: enabled })
    compatibilityProxy.value = enabled
    await check()
    return true
  } catch (e) {
    proxySwitchError.value = String(e)
    await check().catch(() => {})
    return false
  } finally {
    proxySwitchBusy.value = false
  }
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
    skipLauncher,
    skipLauncherAvailable,
    skipLauncherBusy,
    skipLauncherError,
    compatibilityProxy,
    proxySwitchBusy,
    proxySwitchError,
    readinessActions,
    installActions,
    needInstall,
    needFix,
    hasWarnings,
    isReady,
    check,
    refreshSkipLauncher,
    setSkipLauncher,
    setCompatibilityProxy,
    onInstalled,
  }
}
check();
