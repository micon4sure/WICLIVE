<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import Titlebar from './components/Titlebar.vue'
import ReadinessCard from './components/ReadinessCard.vue'
import Config from './components/Config.vue'
import Maps from './components/Maps.vue'

const titlebar = ref<InstanceType<typeof Titlebar>>()
const activeTab = ref<'maps' | 'config'>('maps')
const wasFixed = ref(false)

const mode = computed(() => titlebar.value?.mode ?? 'checking')

const tabsDetached = ref(false)
const appBody = ref<HTMLElement>()

function onScroll() {
  if (!appBody.value) return
  tabsDetached.value = appBody.value.scrollTop > 16
}

onMounted(() => appBody.value?.addEventListener('scroll', onScroll, { passive: true }))
onUnmounted(() => appBody.value?.removeEventListener('scroll', onScroll))
</script>

<template>
  <Titlebar ref="titlebar" />
  <div ref="appBody" class="app-body">
    <ReadinessCard
      v-if="mode !== 'ready' || wasFixed"
      :class="{ 'readiness-full': mode !== 'ready' }"
      :mode="mode"
      :actions="titlebar?.actions ?? []"
      @fixed="wasFixed = true"
    />
    <div v-if="mode === 'ready'" class="tab-area">
      <div class="tab-nav-sticky" :class="{ detached: tabsDetached }">
        <div class="tab-nav">
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'maps' }"
            @click="activeTab = 'maps'"
          >
            <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="1 6 1 22 8 18 16 22 23 18 23 2 16 6 8 2 1 6"/><line x1="8" y1="2" x2="8" y2="18"/><line x1="16" y1="6" x2="16" y2="22"/></svg>
            Maps
          </button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'config' }"
            @click="activeTab = 'config'"
          >
            <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>
            Config
          </button>
        </div>
      </div>
      <div class="tab-content">
        <Maps v-show="activeTab === 'maps'" />
        <Config v-show="activeTab === 'config'" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-body {
  flex: 1;
  overflow-y: auto;
  padding: 0 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 0;
  position: relative;
}

.app-body::before {
  content: '';
  position: fixed;
  top: 80px;
  left: 80px;
  right: 80px;
  bottom: 80px;
  background: rgba(var(--t3-rgb), .2);
  -webkit-mask: url('./assets/map.svg') no-repeat center center / contain;
  mask: url('./assets/map.svg') no-repeat center center / contain;
  pointer-events: none;
  z-index: 0;
}

.app-body::after {
  content: '';
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
  background-image: repeating-linear-gradient(
    45deg,
    transparent,
    transparent 10px,
    rgba(255, 255, 255, 0.03) 10px,
    rgba(255, 255, 255, 0.03) 11px
  );
}

.app-body > * {
  position: relative;
  z-index: 1;
}

.readiness-full {
  flex: 1;
}

.tab-area {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.tab-nav-sticky {
  position: sticky;
  top: 0;
  z-index: 10;
  background: rgba(var(--bg-rgb), 0.92);
  margin: 0 -20px;
  padding: 15px 20px 0;
  box-shadow: none;
  transition: box-shadow 0.2s ease;
}

.tab-nav-sticky.detached {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.tab-nav {
  display: flex;
  gap: 0;
  background: transparent;
  border-bottom: 1px solid rgba(var(--mg-rgb), 0.4);
}

.tab-icon {
  width: 14px;
  height: 14px;
  vertical-align: -2px;
  margin-right: 6px;
}

.tab-btn {
  font-family: 'Oswald', sans-serif;
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 1px;
  padding: 12px 24px;
  background: transparent;
  border: none;
  color: var(--t2);
  cursor: pointer;
  position: relative;
  transition: color 0.3s ease, background 0.3s ease;
}

.tab-btn::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 2px;
  background: var(--dl-light);
  opacity: 0;
  transition: opacity 0.3s ease;
  box-shadow: 0 -2px 8px rgba(var(--dl-light-rgb), 0.4);
}

.tab-btn:hover {
  color: var(--dl-light);
  background: rgba(255, 255, 255, 0.05);
}

.tab-btn.active {
  color: #fff;
  font-weight: 600;
  background: linear-gradient(180deg, #B22222 0%, #8B1A1A 100%);
}

.tab-btn.active::after {
  opacity: 1;
}

.tab-content {
  padding-top: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.tab-content > * {
  flex: 1;
}
</style>
