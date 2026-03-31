<script setup lang="ts">
import { ref } from 'vue'
import Titlebar from './components/Titlebar.vue'
import Config from './components/Config.vue'
import Maps from './components/Maps.vue'

const activeTab = ref<'config' | 'maps'>('config')
</script>

<template>
  <Titlebar />
  <div class="app-body">
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
  border-bottom: 2px solid var(--bd);
}

.tab-btn {
  font-family: 'Oswald', sans-serif;
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  padding: 10px 24px;
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--t3);
  cursor: pointer;
  transition: var(--tr);
  margin-bottom: -2px;
}

.tab-btn:hover {
  color: var(--t2);
}

.tab-btn.active {
  color: var(--sw);
  border-bottom-color: var(--sw);
  text-shadow: 0 0 12px rgba(var(--sw-rgb), 0.3);
}

.tab-content {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-top: 16px;
}
</style>
