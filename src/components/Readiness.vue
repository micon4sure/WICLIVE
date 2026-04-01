<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

type ActionStatus = 'checking' | 'done' | 'needed' | 'applying' | 'error'

interface Action {
  id: string
  label: string
  status: ActionStatus
  detail?: string
  progress?: number
}

const actions = ref<Action[]>([
  { id: 'laa', label: 'LAA Flag', status: 'checking' },
  { id: 'vcredist', label: 'VC++ Redist', status: 'checking' },
  { id: 'patch', label: 'Game Patches', status: 'checking' },
  { id: 'cdkey', label: 'CD Key', status: 'checking' },
  { id: 'hooks', label: 'Proxy (Hooks)', status: 'checking' },
])

const allReady = ref(false)

defineExpose({ allReady })

function find(id: string) {
  return actions.value.find(a => a.id === id)!
}

async function runChecks() {
  // Need install path for everything
  const installPath = await invoke<string | null>('get_install_path')
  if (!installPath) {
    for (const a of actions.value) {
      a.status = 'error'
      a.detail = 'Game not found'
    }
    return
  }

  // LAA
  try {
    const laa = await invoke<boolean>('get_laa_flag')
    const a = find('laa')
    a.status = laa ? 'done' : 'needed'
    a.detail = laa ? 'Enabled' : 'Not set'
  } catch (e) {
    find('laa').status = 'error'
    find('laa').detail = String(e)
  }

  // VC++ Redist
  try {
    const installed = await invoke<boolean>('check_vcredist')
    const a = find('vcredist')
    a.status = installed ? 'done' : 'needed'
    a.detail = installed ? 'Installed' : 'Missing'
  } catch (e) {
    find('vcredist').status = 'error'
    find('vcredist').detail = String(e)
  }

  // Game version (patches)
  try {
    const ver = await invoke<{ major: number; minor: number; patch: number; build: number }>('get_game_version')
    const a = find('patch')
    const versionStr = `${ver.major}.${ver.minor}.${ver.patch}.${ver.build}`
    const upToDate = ver.patch === 1 && ver.build === 1
    a.status = upToDate ? 'done' : 'needed'
    a.detail = upToDate ? versionStr : `${versionStr} → 1.0.1.1`
  } catch (e) {
    find('patch').status = 'error'
    find('patch').detail = String(e)
  }

  // CD Key
  try {
    const key = await invoke<string>('get_cd_key')
    const a = find('cdkey')
    const hasKey = key.length > 0 && key.toLowerCase() !== 'invalid'
    a.status = hasKey ? 'done' : 'needed'
    a.detail = hasKey ? key.substring(0, 9) + '...' : 'Not set'
  } catch (e) {
    find('cdkey').status = 'error'
    find('cdkey').detail = String(e)
  }

  // Hooks
  try {
    const installed = await invoke<boolean>('check_hooks')
    const a = find('hooks')
    if (installed) {
      const version = await invoke<string>('get_hooks_version')
      a.status = 'done'
      a.detail = version.trim()
    } else {
      a.status = 'needed'
      a.detail = 'Not installed'
    }
  } catch (e) {
    find('hooks').status = 'error'
    find('hooks').detail = String(e)
  }

  // All ready?
  allReady.value = actions.value.every(a => a.status === 'done')
}

onMounted(runChecks)

function statusIcon(status: ActionStatus): string {
  switch (status) {
    case 'done': return '\u2713'
    case 'needed': return '\u25CB'
    case 'applying': return '\u25CF'
    case 'error': return '\u2717'
    case 'checking': return '\u00B7\u00B7\u00B7'
  }
}
</script>

<template>
  <div class="readiness">
    <div class="action-list">
      <div
        v-for="action in actions"
        :key="action.id"
        class="action-row"
        :class="'status-' + action.status"
      >
        <span class="action-icon">{{ statusIcon(action.status) }}</span>
        <span class="action-label">{{ action.label }}</span>
        <span v-if="action.detail" class="action-detail">{{ action.detail }}</span>
        <div
          v-if="action.status === 'applying' && action.progress != null"
          class="action-progress"
        >
          <div class="action-progress-bar" :style="{ width: action.progress + '%' }" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.readiness {
  background: var(--grad-card);
  border: 1px solid var(--bd);
  padding: 16px 20px;
  flex-shrink: 0;
}

.readiness-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.readiness-header h2 {
  margin: 0;
  font-size: 1.25rem;
}

.action-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.action-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(var(--s1-rgb), 0.6);
  border-left: 3px solid transparent;
  font-family: 'Rajdhani', sans-serif;
  font-size: 15px;
  font-weight: 500;
}

.action-row.status-done {
  border-left-color: var(--g);
}

.action-row.status-needed {
  border-left-color: var(--sw);
}

.action-row.status-applying {
  border-left-color: var(--sw);
}

.action-row.status-error {
  border-left-color: var(--dl-light);
}

.action-row.status-checking {
  border-left-color: var(--t3);
}

.action-icon {
  width: 20px;
  text-align: center;
  font-weight: 700;
  font-size: 14px;
}

.status-done .action-icon { color: var(--g); }
.status-needed .action-icon { color: var(--sw); }
.status-applying .action-icon { color: var(--sw); }
.status-error .action-icon { color: var(--dl-light); }
.status-checking .action-icon { color: var(--t3); }

.action-label {
  font-family: 'Oswald', sans-serif;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-size: 13px;
}

.action-detail {
  margin-left: auto;
  color: var(--t3);
  font-size: 13px;
}

.action-progress {
  flex: 1;
  height: 4px;
  background: rgba(var(--mg-rgb), 0.3);
  margin-left: auto;
  max-width: 200px;
}

.action-progress-bar {
  height: 100%;
  background: var(--sw);
  transition: width 0.3s ease;
}
</style>
