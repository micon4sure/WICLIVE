import { reactive, ref, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api'
import axios from 'axios'
import type { Job, Job_Data } from './wic-job'

// Singleton state — shared across all consumers
const jobsInit: Job[] = reactive([])
const isInitialized = ref(false)
const initializeSuccess = ref(true)

const needsCDKey = ref(false)
const needsVCRedist = ref(false)
const needsPatch = ref(false)
const needsHooks = ref(false)
const needsHooksUpdate = ref(false)
const needsLAA = ref(false)
const needsAction = ref(false)

const installDir = ref('')
const gameVersion = ref<string | null>(null)
const hooksVersion = ref('')
const isSoviet = ref(false)
const cdKey = ref('')

let _resolveInit: (() => void) | null = null

const checkInit = () => {
  for (const job of jobsInit) {
    if (job.data.status === 'queued' || job.data.status === 'running') return
  }
  isInitialized.value = true
  onInitialized()
  if (_resolveInit) {
    _resolveInit()
    _resolveInit = null
  }
}

const createInitJob = (title: string, action: (helpers: { addInfo: (text: string, highlight?: boolean) => void }) => Promise<void>) => {
  const data: Job_Data = reactive({
    title,
    status: 'queued',
    info: [],
    progress: 0
  })

  const run = async () => {
    data.status = 'running'
    const addInfo = (text: string, highlight: boolean = false) => {
      data.info.push({ text, highlight })
    }
    try {
      await action({ addInfo })
      data.status = 'success'
    } catch (e) {
      data.info.push({ text: String(e), highlight: true })
      data.status = 'error'
      initializeSuccess.value = false
    } finally {
      checkInit()
    }
  }

  const job = { data, run }
  jobsInit.push(job)
  return job
}

const initSetupState = async () => {
  while (jobsInit.length > 0) jobsInit.pop()
  isInitialized.value = false
  initializeSuccess.value = true

  await nextTick()

  createInitJob("Check game patches", async ({ addInfo }) => {
    const version = await invoke('extract_game_version') as any
    gameVersion.value = `${version.major}.${version.minor}.${version.patch}.${version.build}`
    addInfo(gameVersion.value)
    needsPatch.value = version.patch !== 1 || version.build !== 1
    if (needsPatch.value) addInfo('Not patched to latest version', true)
  }).run()

  createInitJob("Check LAA flag", async ({ addInfo }) => {
    const laaFlag = await invoke('get_laa_flag') as boolean
    needsLAA.value = !laaFlag
    addInfo(laaFlag ? 'Enabled' : 'Not set', needsLAA.value)
  }).run()

  createInitJob('Check VC++ Redistributable', async ({ addInfo }) => {
    needsVCRedist.value = await invoke('needs_vc_redist')
    addInfo(needsVCRedist.value ? 'Not installed' : 'Installed', needsVCRedist.value)
  }).run()

  createInitJob("Check CD key", async ({ addInfo }) => {
    const key = cdKey.value = await invoke('get_cd_key')
    needsCDKey.value = !key || key === "invalid"
    addInfo(needsCDKey.value ? 'Missing or invalid' : key, needsCDKey.value)
    needsVCRedist.value = await invoke('needs_vc_redist')
  }).run()

  createInitJob('Check multiplayer fix', async ({ addInfo }) => {
    needsHooks.value = await invoke('needs_hooks')
    hooksVersion.value = (await axios.get('https://www.wicgate.com/wic_cl_hook-version.txt')).data
    needsHooksUpdate.value = !needsHooks.value && await invoke('needs_hooks_update', { version: hooksVersion.value })
    if (needsHooks.value) addInfo('Not installed', true)
    else if (needsHooksUpdate.value) addInfo(`Update available: ${hooksVersion.value}`, true)
    else addInfo(hooksVersion.value)
  }).run()
}

const onInitialized = async () => {
  needsAction.value = needsHooks.value || needsHooksUpdate.value || needsCDKey.value || needsVCRedist.value || needsLAA.value || needsPatch.value

  installDir.value = await invoke('get_install_path')
  isSoviet.value = await invoke('is_soviet_assault')
}

const ensureInit = (): Promise<void> => {
  if (isInitialized.value) return Promise.resolve()
  return new Promise(resolve => { _resolveInit = resolve })
}

export default function useSetupState() {
  return {
    jobsInit,
    isInitialized,
    initializeSuccess,
    needsCDKey,
    needsVCRedist,
    needsPatch,
    needsHooks,
    needsHooksUpdate,
    needsLAA,
    needsAction,
    installDir,
    gameVersion,
    hooksVersion,
    isSoviet,
    cdKey,
    initSetupState,
    ensureInit,
  }
}
