<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface RemoteMap {
  name: string
  size: number
  hash: string
  date: string
  uploader: string
  version: number
  beta: number
  final: number
}

type MapStatus = 'missing' | 'outdated' | 'current' | 'loading' | 'pending' | 'error'

interface MapEntry {
  name: string
  remote: RemoteMap
  status: MapStatus
  error: string
}

const maps = ref<MapEntry[]>([])
const loading = ref(true)
const loadError = ref('')
const downloading = ref<string | null>(null)
const downloadQueue = ref<string[]>([])

const dlDownloaded = ref(0)
const dlTotal = ref(0)
let lastDlUpdate = 0

const statusOrder: Record<MapStatus, number> = {
  loading: 0,
  error: 1,
  missing: 2,
  outdated: 3,
  pending: 4,
  current: 5,
}

const gameTypes: Record<string, string> = {
  do: 'Domination',
  as: 'Assault',
  tw: 'Tug of War',
}

function parseGameType(name: string): string {
  const prefix = name.split('_')[0]
  return gameTypes[prefix] || prefix.toUpperCase()
}

function displayName(name: string): string {
  return name
    .replace(/\.sdf$/, '')
    .replace(/^(do|as|tw)_/, '')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, c => c.toUpperCase())
}

function formatSize(bytes: number): string {
  const mb = bytes / (1024 * 1024)
  return `${mb.toFixed(1)} MB`
}

function formatDate(date: string): string {
  if (!date) return ''
  return date.split(' ')[0]
}

function dlPct(): number {
  if (dlTotal.value <= 0) return 0
  return Math.min(100, Math.round(dlDownloaded.value / dlTotal.value * 100))
}

function dlText(): string {
  if (dlTotal.value <= 0) return '0%'
  const pct = dlPct()
  if (pct >= 100) return 'Verifying...'
  return `${pct}%`
}

const sorted = computed(() => {
  return [...maps.value].sort((a, b) => {
    const sd = statusOrder[a.status] - statusOrder[b.status]
    if (sd !== 0) return sd
    return a.name.localeCompare(b.name)
  })
})

const needsDownload = computed(() =>
  maps.value.filter(m => m.status === 'missing' || m.status === 'outdated' || m.status === 'error')
)

const isDownloading = computed(() => downloading.value !== null || downloadQueue.value.length > 0)
const isDev = import.meta.env.DEV

async function loadMaps() {
  loading.value = true
  loadError.value = ''
  try {
    const apiUrl = await invoke<string>('get_api_url')
    const resp = await fetch(`${apiUrl}/maps/data`)
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`)
    const remote: Record<string, RemoteMap> = await resp.json()

    let localFiles: string[] = []
    try { localFiles = await invoke('list_map_files') } catch {}
    const localSet = new Set(localFiles.map(f => f.toLowerCase()))

    const entries = await Promise.all(Object.values(remote).map(async (info) => {
      const key = info.name.toLowerCase()
      let status: MapStatus = 'missing'

      if (localSet.has(key)) {
        try {
          const localHash = await invoke<string>('get_map_hash', { filename: info.name })
          status = localHash === info.hash ? 'current' : 'outdated'
        } catch {
          status = 'missing'
        }
      }

      return { name: info.name, remote: info, status, error: '' } as MapEntry
    }))

    maps.value = entries
  } catch (e) {
    loadError.value = String(e)
  } finally {
    loading.value = false
  }
}

async function downloadSingle(filename: string) {
  const entry = maps.value.find(m => m.name === filename)
  if (!entry || entry.status === 'current' || entry.status === 'loading') return
  if (entry.status !== 'pending') entry.error = ''

  entry.status = 'loading'
  entry.error = ''
  downloading.value = filename
  dlDownloaded.value = 0
  dlTotal.value = 0
  lastDlUpdate = 0

  const unlisten = await listen('map-progress', (e: any) => {
    const p = e.payload
    if (p.stage === 'downloading') {
      const now = Date.now()
      if (p.downloaded < p.total && now - lastDlUpdate < 200) return
      lastDlUpdate = now
      dlDownloaded.value = p.downloaded
      dlTotal.value = p.total
    }
  })

  try {
    await invoke('download_map', { filename })
    const hash = await invoke<string>('get_map_hash', { filename })
    console.log(`[hash] ${filename} local=${hash} remote=${entry.remote.hash}`)
    if (hash === entry.remote.hash) {
      entry.status = 'current'
    } else {
      entry.status = 'error'
      entry.error = `Hash mismatch: local=${hash} remote=${entry.remote.hash}`
    }
  } catch (e) {
    entry.status = 'error'
    entry.error = String(e)
  } finally {
    unlisten()
    downloading.value = null
  }
}

async function downloadAll() {
  // Build queue in display order (top to bottom)
  const queue = sorted.value
    .filter(m => m.status === 'missing' || m.status === 'outdated' || m.status === 'error')
    .map(m => m.name)

  // Mark all queued as pending
  for (const name of queue) {
    const entry = maps.value.find(m => m.name === name)
    if (entry) {
      entry.status = 'pending'
      entry.error = ''
    }
  }

  downloadQueue.value = [...queue]
  for (const filename of queue) {
    await downloadSingle(filename)
    downloadQueue.value = downloadQueue.value.filter(f => f !== filename)
  }
  downloadQueue.value = []
}

async function deleteAll() {
  await invoke('delete_all_maps')
  await loadMaps()
}

const showUpload = ref(false)
const uploadFile = ref<HTMLInputElement>()
const uploadKey = ref(localStorage.getItem('upload-key') ?? '')
const fileName = ref('')

function onFileChange() {
  fileName.value = uploadFile.value?.files?.[0]?.name ?? ''
}
const uploading = ref(false)
const uploadStatus = ref('')

async function uploadMap() {
  const file = uploadFile.value?.files?.[0]
  if (!file) { uploadStatus.value = 'No file selected'; return }
  if (!uploadKey.value) { uploadStatus.value = 'No API key'; return }

  uploading.value = true
  uploadStatus.value = `Uploading ${file.name}...`

  const apiUrl = await invoke<string>('get_api_url')
  const form = new FormData()
  form.append('file', file)
  form.append('key', uploadKey.value)

  try {
    const resp = await fetch(`${apiUrl}/maps/upload`, { method: 'POST', body: form })
    if (!resp.ok) throw new Error(await resp.text())
    localStorage.setItem('upload-key', uploadKey.value)
    uploadStatus.value = 'Upload complete'
    showUpload.value = false
    uploadStatus.value = ''
    await loadMaps()
  } catch (e) {
    uploadStatus.value = String(e)
  } finally {
    uploading.value = false
  }
}

onMounted(async () => {
  await loadMaps()
})
</script>

<template>
  <div class="panel">
    <!-- Panel header -->
    <div class="panel-header">
      <div class="panel-header-top">
        <div>
          <h2 class="panel-title">Maps</h2>
          <p class="panel-subtitle">{{ maps.length }} maps<template v-if="needsDownload.length"> &middot; {{ needsDownload.length }} need{{ needsDownload.length === 1 ? 's' : '' }} download</template></p>
        </div>
        <div class="panel-header-actions">
          <button
            v-if="isDev"
            class="btn-panel"
            :disabled="isDownloading"
            @click="showUpload = !showUpload"
          >
            Upload Map
          </button>
          <button
            v-if="isDev && maps.some(m => m.status === 'current')"
            class="btn-panel"
            :disabled="isDownloading"
            @click="deleteAll"
          >
            Delete All
          </button>
          <button
            v-if="needsDownload.length"
            class="btn-panel btn-panel-primary"
            :disabled="isDownloading"
            @click="downloadAll"
          >
            {{ isDownloading ? 'Downloading...' : 'Download All' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="loading" class="panel-empty">Loading...</div>
    <div v-else-if="loadError" class="panel-empty panel-empty-error">{{ loadError }}</div>

    <table v-else class="map-table">
      <thead>
        <tr class="table-header">
          <th class="th-status" />
          <th class="th-name">Name</th>
          <th class="th-file">File</th>
          <th class="th-type">Type</th>
          <th class="th-size">Size</th>
          <th class="th-uploader">Uploader</th>
          <th class="th-date">Date</th>
          <th class="th-action" />
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(map, i) in sorted"
          :key="map.name"
          class="table-row"
          :class="[`is-${map.status}`, { odd: i % 2 === 0 }]"
        >
          <!-- Error row -->
          <template v-if="map.status === 'error'">
            <td class="cell-status"><span class="status-dot" /></td>
            <td class="cell-name">{{ displayName(map.name) }}<span v-if="map.remote?.version > 1" class="version-tag">v{{ map.remote.version }}</span></td>
            <td class="cell-error" colspan="5">{{ map.error }}</td>
            <td class="cell-action">
              <button class="btn-download" :disabled="isDownloading" @click="downloadSingle(map.name)">Retry</button>
            </td>
          </template>

          <!-- Loading row -->
          <template v-else-if="map.status === 'loading'">
            <td class="cell-status"><span class="status-dot" /></td>
            <td class="cell-name">{{ displayName(map.name) }}<span v-if="map.remote?.version > 1" class="version-tag">v{{ map.remote.version }}</span></td>
            <td class="cell-progress" colspan="6">
              <span class="loading-pct">{{ dlText() }}</span>
              <div v-if="dlTotal > 0" class="progress-track">
                <div class="progress-fill" :style="{ width: dlPct() + '%' }" />
              </div>
            </td>
          </template>

          <!-- Normal row -->
          <template v-else>
            <td class="cell-status"><span class="status-dot" /></td>
            <td class="cell-name">{{ displayName(map.name) }}<span v-if="map.remote.version > 1" class="version-tag">v{{ map.remote.version }}</span></td>
            <td class="cell-file">{{ map.name }}</td>
            <td class="cell-type">{{ parseGameType(map.name) }}</td>
            <td class="cell-size">{{ formatSize(map.remote.size) }}</td>
            <td class="cell-uploader">{{ map.remote.uploader }}</td>
            <td class="cell-date">{{ formatDate(map.remote.date) }}</td>
            <td class="cell-action">
              <span v-if="map.status === 'current'" class="action-check">&#10003;</span>
              <span v-else-if="map.status === 'pending'" class="action-pending">&middot;&middot;</span>
              <button
                v-else
                class="btn-download"
                :disabled="isDownloading"
                @click="downloadSingle(map.name)"
              >
                {{ map.status === 'outdated' ? 'Update' : 'Get' }}
              </button>
            </td>
          </template>
        </tr>
      </tbody>
    </table>

    <!-- Upload modal -->
    <Teleport to="body">
      <div v-if="showUpload" class="modal-overlay" @click.self="showUpload = false">
        <div class="modal-box">
          <div class="modal-header">
            <span class="modal-title">Upload Map</span>
            <button class="modal-close" @click="showUpload = false">&times;</button>
          </div>
          <div class="modal-body">
            <label class="field-label">Map File (.sdf)</label>
            <input ref="uploadFile" type="file" accept=".sdf" class="file-hidden" @change="onFileChange" />
            <div class="file-pick" @click="uploadFile?.click()">
              <span v-if="fileName" class="file-name">{{ fileName }}</span>
              <span v-else class="file-placeholder">Browse...</span>
            </div>
            <label class="field-label">API Key</label>
            <input v-model="uploadKey" type="text" placeholder="API Key" class="field-input" />
            <div v-if="uploadStatus" class="upload-status">{{ uploadStatus }}</div>
          </div>
          <div class="modal-footer">
            <button class="btn-panel" @click="showUpload = false">Cancel</button>
            <button class="btn-panel btn-panel-primary" :disabled="uploading" @click="uploadMap">
              {{ uploading ? 'Uploading...' : 'Upload' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
/* ── Panel shell ─────────────────────────────────────── */
.panel {
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: linear-gradient(135deg, rgba(18, 30, 39, 0.85) 0%, rgba(9, 15, 20, 0.85) 100%);
  box-shadow: var(--shadow-panel);
  overflow: hidden;
}

/* ── Panel header ────────────────────────────────────── */
.panel-header {
  background:
    radial-gradient(circle, rgba(255, 255, 255, 0.06) 1px, transparent 1px),
    linear-gradient(180deg, #1a3a5c 0%, #0d1f36 100%);
  background-size: 4px 4px, 100% 100%;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(var(--t3-rgb), 0.2);
}

.panel-header-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.panel-title {
  margin: 0;
  font-family: 'Oswald', sans-serif;
  font-size: 18px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #fff;
  text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
}

.panel-subtitle {
  margin: 4px 0 0;
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  font-weight: 400;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--t2);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
}

.panel-header-actions {
  display: flex;
  gap: 6px;
}

.btn-panel {
  font-family: 'Oswald', sans-serif;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 6px 16px;
  background: rgba(var(--mg-rgb), 0.4);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: var(--t2);
  cursor: pointer;
  transition: var(--tr);
}
.btn-panel:hover:not(:disabled) {
  background: linear-gradient(0deg, rgba(255, 255, 255, 0.2) 0%, rgba(255, 255, 255, 0.08) 100%);
  border-color: rgba(255, 255, 255, 0.3);
  color: #fff;
}
.btn-panel:disabled {
  opacity: 0.4;
  cursor: default;
}
.btn-panel-primary {
  background: rgba(var(--gold-rgb), 0.15);
  border-color: rgba(var(--gold-rgb), 0.4);
  color: var(--gold);
}
.btn-panel-primary:hover:not(:disabled) {
  background: linear-gradient(180deg, var(--gold-bright) 0%, var(--gold-dark) 100%);
  color: var(--ink);
  border-color: var(--gold);
}

/* ── Panel empty/loading ─────────────────────────────── */
.panel-empty {
  padding: 40px;
  text-align: center;
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  color: var(--t3);
}
.panel-empty-error { color: var(--dl-light); }

/* ── Table ───────────────────────────────────────────── */
.map-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
}

.table-header th {
  padding: 8px 0;
  background: linear-gradient(180deg, rgba(31, 49, 61, 0.5) 0%, rgba(18, 30, 39, 0.5) 100%);
  border-bottom: 1px solid rgba(var(--mg-rgb), 0.25);
  font-family: 'Oswald', sans-serif;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--t2);
  text-align: left;
}

.th-status { width: 22px; }
.th-name { }
.th-file { width: 180px; }
.th-type { width: 80px; }
.th-size { width: 70px; }
.th-uploader { width: 100px; }
.th-date { width: 110px; }
.th-action { width: 70px; text-align: center; }

/* ── Table rows ──────────────────────────────────────── */
.table-row td {
  padding: 0;
  height: 40px;
  vertical-align: middle;
  background: linear-gradient(90deg, rgba(24, 38, 48, 0.5) 0%, rgba(14, 24, 31, 0.5) 100%);
  border-bottom: 1px solid rgba(var(--mg-rgb), 0.15);
  transition: background 0.2s;
}

.table-row.odd td {
  background: linear-gradient(90deg, rgba(18, 30, 39, 0.5) 0%, rgba(9, 15, 20, 0.5) 100%);
}

.table-row:hover td {
  background: linear-gradient(90deg, rgba(31, 49, 61, 0.5) 0%, rgba(20, 34, 42, 0.5) 100%);
}

/* ── Status dot ──────────────────────────────────────── */
.cell-status {
  text-align: center;
}

.status-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--t3);
}

.is-current .status-dot { background: var(--g); }
.is-missing .status-dot { background: var(--t3); }
.is-outdated .status-dot { background: var(--sw); }
.is-loading .status-dot { background: var(--b); }
.is-pending .status-dot { background: var(--t3); }
.is-error .status-dot { background: var(--dl-light); }

/* ── Cells ───────────────────────────────────────────── */
.cell-name {
  padding: 0 12px;
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  letter-spacing: 0.3px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.version-tag {
  margin-left: 6px;
  font-size: 11px;
  font-weight: 400;
  color: var(--t3);
}

.cell-size,
.cell-uploader,
.cell-date {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  color: var(--t3);
  white-space: nowrap;
  padding: 0 8px;
}

.cell-file {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  color: var(--t2);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 12px;
}

.cell-type {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  color: var(--t3);
  white-space: nowrap;
}

.cell-action {
  text-align: center;
}

.action-check {
  font-size: 14px;
  color: var(--g);
  font-weight: 700;
}

.action-pending {
  font-size: 14px;
  color: var(--t3);
  letter-spacing: 2px;
}

.btn-download {
  width: 100%;
  height: 100%;
  font-family: 'Rajdhani', sans-serif;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0;
  background: rgba(var(--gold-rgb), 0.12);
  color: var(--gold);
  border: 1px solid rgba(var(--gold-rgb), 0.25);
  cursor: pointer;
  transition: var(--tr);
}
.btn-download:hover:not(:disabled) {
  background: linear-gradient(180deg, var(--gold-bright) 0%, var(--gold-dark) 100%);
  color: var(--ink);
  border-color: var(--gold);
}
.btn-download:disabled {
  opacity: 0.4;
  cursor: default;
}

/* ── Error row ───────────────────────────────────────── */
.cell-error {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  font-weight: 600;
  color: var(--dl-light);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 12px;
}

/* ── Loading row ─────────────────────────────────────── */
.cell-progress {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 12px;
  min-width: 0;
}

.loading-pct {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  font-weight: 600;
  color: var(--b);
}

.progress-track {
  width: 100%;
  height: 4px;
  background: rgba(var(--mg-rgb), 0.4);
  overflow: hidden;
}
.progress-fill {
  height: 100%;
  background: var(--b);
  transition: width 0.15s linear;
}

/* ── Upload modal ──────────────────────────────────── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-box {
  width: 380px;
  background: linear-gradient(135deg, #121e27 0%, #090f14 100%);
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  background: linear-gradient(180deg, #1a3a5c 0%, #0d1f36 100%);
  border-bottom: 1px solid rgba(var(--t3-rgb), 0.2);
}

.modal-title {
  font-family: 'Oswald', sans-serif;
  font-size: 16px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #fff;
}

.modal-close {
  background: none;
  border: none;
  color: var(--t3);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  transition: color 0.2s;
}
.modal-close:hover { color: #fff; }

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-label {
  font-family: 'Oswald', sans-serif;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--t2);
}

.file-hidden {
  display: none;
}

.file-pick {
  padding: 8px 12px;
  background: rgba(var(--mg-rgb), 0.2);
  border: 1px solid rgba(var(--mg-rgb), 0.4);
  cursor: pointer;
  transition: border-color 0.2s;
}
.file-pick:hover {
  border-color: rgba(var(--b-rgb), 0.6);
}

.file-name {
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  color: var(--t);
}

.file-placeholder {
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  color: var(--t3);
}

.field-input {
  font-family: 'Rajdhani', sans-serif;
  font-size: 14px;
  padding: 8px 12px;
  background: rgba(var(--mg-rgb), 0.2);
  border: 1px solid rgba(var(--mg-rgb), 0.4);
  color: var(--t);
  outline: none;
  transition: border-color 0.2s;
}
.field-input:focus {
  border-color: rgba(var(--b-rgb), 0.6);
}

.upload-status {
  font-family: 'Rajdhani', sans-serif;
  font-size: 13px;
  color: var(--t3);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid rgba(var(--mg-rgb), 0.25);
}
</style>
