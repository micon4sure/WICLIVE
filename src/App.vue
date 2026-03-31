<script setup lang="ts">
import { ref, computed } from 'vue'
import Titlebar from './components/Titlebar.vue'
import ReadinessCard from './components/ReadinessCard.vue'
import Config from './components/Config.vue'
import Maps from './components/Maps.vue'

const titlebar = ref<InstanceType<typeof Titlebar>>()
const activeTab = ref<'config' | 'maps'>('config')

const showReadiness = computed(() => {
  const acts = titlebar.value?.actions
  if (!acts) return false
  return acts.some(a => a.status !== 'checking' && a.status !== 'done')
})
</script>

<template>
  <Titlebar ref="titlebar" />
  <div class="app-body">
    <ReadinessCard
      v-if="showReadiness && titlebar?.actions"
      :actions="titlebar.actions"
    />
    <div class="tab-area">
      <div class="tab-nav">
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'config' }"
          @click="activeTab = 'config'"
        >
          Config
        </button>
        <button
          class="tab-btn"
          :class="{ active: activeTab === 'maps' }"
          @click="activeTab = 'maps'"
        >
          Maps
        </button>
      </div>
      <div class="tab-content">
        <Config v-if="activeTab === 'config'" />
        <Maps v-if="activeTab === 'maps'" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px 20px;
  gap: 16px;
}

.tab-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.tab-nav {
  display: flex;
  gap: 0;
  background: linear-gradient(180deg, rgba(var(--graphite-rgb), 0.95) 0%, rgba(var(--graphite-dark-rgb), 0.95) 100%);
  border-bottom: 2px solid rgba(var(--graphite-rgb), 0.8);
}

.tab-btn {
  font-family: 'Oswald', sans-serif;
  font-size: 14px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 12px 24px;
  background: linear-gradient(180deg, rgba(var(--graphite-rgb), 0.9) 0%, rgba(var(--graphite-dark-rgb), 0.92) 100%);
  border: 1px solid rgba(var(--graphite-dark-rgb), 0.6);
  color: var(--t2);
  cursor: pointer;
  transition: var(--tr);
}

.tab-btn:hover {
  color: var(--t);
  background: linear-gradient(180deg, rgba(var(--graphite-rgb), 1) 0%, rgba(var(--graphite-dark-rgb), 1) 100%);
}

.tab-btn.active {
  background: linear-gradient(180deg, rgba(var(--sw-rgb), 0.98) 0%, rgba(var(--sw-rgb), 0.8) 100%);
  color: var(--ink);
  font-weight: 600;
  border-color: rgba(var(--sw-rgb), 0.95);
  box-shadow: 0 0 18px rgba(var(--sw-rgb), 0.35);
}

.tab-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-top: 16px;
}
</style>
