<script setup lang="ts">
import { invoke } from '@tauri-apps/api';
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router';
import useSetupState from './lib/use-setup-state'

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';
import iconXMark from '@fortawesome/fontawesome-free/svgs/solid/xmark.svg';
import iconWarning from '@fortawesome/fontawesome-free/svgs/solid/triangle-exclamation.svg';

const { jobsInit, initSetupState, initializeSuccess } = useSetupState()

const _version = ref('');
onMounted(async () => {
  const config: any = await invoke('get_config')
  _version.value = config.VERSION
})

const router = useRouter()
const home = router.resolve('/').href

console.log(router.currentRoute.value.path)
onMounted(async () => {
  if (localStorage.getItem('do-install')) {
    router.push('/install/goes')
    return;
  }

  if (localStorage.getItem('initialized')) {
    return;
  }

  const installPath = await invoke('get_install_path')
  if (!installPath) {
    router.push('/init/game')
    return;
  }

  let version;
  try {
    version = await invoke('extract_game_version') as any;
  } catch (error) {
    console.log('forwarding to broken')
    router.push('/init/broken')
    return
  }
  const isPatched = version.patch == 1 && version.build == 1;
  if (!isPatched) {
    router.push('/init/patch')
  }
})

const isInstalled = ref(false);
onMounted(async () => {
  const installPath = await invoke('get_install_path')
  if (installPath) {
    isInstalled.value = true;
    initSetupState()
  }
})

const startGame = () => {
  invoke('start_game')
}

const hasHighlight = (job) => {
  return job.data.info.some(i => i.highlight)
}

const pillValue = (job) => {
  if (!job.data.info.length) return ''
  return job.data.info[0].text
}

const pillOrder = (job) => {
  if (job.data.status === 'error') return 0
  if (job.data.status === 'success' && hasHighlight(job)) return 1
  if (job.data.status === 'queued' || job.data.status === 'running') return 2
  return 3
}

const sortedJobs = computed(() => {
  return [...jobsInit].sort((a, b) => pillOrder(a) - pillOrder(b))
})
</script>

<template>
  <h1>
    <a :href="home"><img src="./assets/wiclive.png" alt="WIC LIVE" /> <small>{{ _version }}</small></a>
    <p v-if="isInstalled"><button class="btn cta special" @click="startGame">Start game</button></p>
  </h1>
  <div id="status-bar" v-if="isInstalled">
    <div
      v-for="(job, idx) in sortedJobs"
      :key="idx"
      :class="['status-pill', {
        'checking': job.data.status === 'queued' || job.data.status === 'running',
        'ok': job.data.status === 'success' && !hasHighlight(job),
        'warn': job.data.status === 'success' && hasHighlight(job),
        'err': job.data.status === 'error'
      }]"
      :title="job.data.title"
    >
      <span class="pill-icon">
        <span class="spinner-border spinner-border-sm" v-if="job.data.status === 'queued' || job.data.status === 'running'"></span>
        <iconCheck v-else-if="job.data.status === 'success' && !hasHighlight(job)" />
        <iconWarning v-else-if="job.data.status === 'success' && hasHighlight(job)" />
        <iconXMark v-else-if="job.data.status === 'error'" />
      </span>
      <span class="pill-label">{{ job.data.title.replace('Check ', '') }}</span>
      <span class="pill-value" v-if="pillValue(job)">{{ pillValue(job) }}</span>
    </div>
    <div class="status-pill err" v-if="!initializeSuccess">
      <span class="pill-icon"><iconXMark /></span>
      <span class="pill-label">Init failed</span>
    </div>
  </div>
  <div id="container">
    <router-view />
  </div>
  <footer>
    This project is not affiliated, associated, authorized, endorsed by, or in any way officially connected with MASSIVE
    Entertainment or UBISOFT, or any of their subsidiaries or affiliates.<br />
    All trademarks and registered trademarks are the property of their respective owners. The use of these names,
    trademarks, and brands does not imply endorsement.
  </footer>
</template>

<style lang="scss">
@font-face {
  font-family: "eurostib";
  src: url("./assets/eurostib.ttf");
}

@font-face {
  font-family: "eurostext";
  src: url("./assets/ESTEXTR.ttf");
}

@font-face {
  font-family: "DIN_1451";
  src: url("./assets/bahnschrift.ttf");
  font-weight: 100;
}

@font-face {
  font-family: "Roboto";
  src: url("./assets/Roboto.ttf");
  font-weight: 200;
}

@font-face {
  font-family: "Orbitron";
  src: url("./assets/Orbitron.ttf");
  font-weight: 100;
}

body {
  background: url("./assets/pattern-stripes.svg");
}

#app {
  background: url("./assets/map.svg") no-repeat center center fixed;
}

h1 {
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin-bottom: 0;
  padding: 25px;
  display: flex;

  p {
    span.cta {
      display: inline-block;
      align-items: center;
    }

    flex: 1;
    display: flex;
    justify-content: right;
    align-items: flex-end;
    margin: 0;
    padding: 0;
  }

  a {
    display: flex;
    align-items: flex-end;
    text-decoration: none;
  }
}


h1 small {
  margin-left: 10px;
  font-size: 12px;
  color: #fff;
}

// STATUS BAR
#status-bar {
  display: flex;
  align-items: stretch;
  margin-bottom: 15px;
  background: linear-gradient(180deg, rgba(5, 30, 45, 0.85) 0%, rgba(5, 50, 70, 0.6) 100%);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  border-bottom: 1px solid rgba(5, 84, 121, 0.5);
}

.status-pill {
  flex: 1;
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 8px;
  padding: 12px 16px;
  font-family: "DIN_1451";
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  font-stretch: 110%;
  white-space: nowrap;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: inset -1px 0 0 rgba(0, 0, 0, 0.3);
  color: #667;

  &:last-child {
    border-right: none;
    box-shadow: none;
  }

  .pill-icon {
    display: flex;
    align-self: center;

    svg {
      width: 14px;
      height: 14px;
      fill: currentColor;
    }

    .spinner-border-sm {
      width: 14px;
      height: 14px;
      border-width: 2px;
      color: #556;
    }
  }

  .pill-label {
    letter-spacing: 1.5px;
    color: rgba(255, 255, 255, 0.55);
  }

  .pill-value { }

  &.checking {
    color: #556;
    .pill-value { color: #889; }
  }

  &.ok {
    color: #3a9a5c;
    .pill-value { color: #4aba6e; }
  }

  &.warn {
    color: #cc4400;
    .pill-value { color: #e05520; }
  }

  &.err {
    color: #cc2222;
    .pill-value { color: #e03333; }
  }
}

h2 {
  background: linear-gradient(to right, #055479 0%, transparentize(#ce2e06, 1) 50%);
  border-top-left-radius: 5px;
  border-bottom-left-radius: 5px;
  padding: 10px 15px;

  &,
  * {
    font-family: Orbitron;
    font-size: 32px;
    letter-spacing: 1px;
    text-transform: uppercase;
    font-stretch: 80%;
  }
}

* {
  font-family: "Roboto";
}

#container {
  padding: 20px;
  margin: 0px;
  width: calc(100vw - 20px);
  min-height: calc(100vh - 75px);
  flex: 1;
}

.card {
  background: transparentize($color: #fff, $amount: .8);
}

.card-header {
  font-family: Orbitron;
  font-size: 26px;
  padding-left: 20px;
  letter-spacing: 1px;
  text-transform: uppercase;
  font-stretch: 80%;
  background: transparentize($color: #000, $amount: .5);

  &,
  & * {
    font-family: Orbitron;
  }
}

.cta {
  font-family: "DIN_1451";
  letter-spacing: 3px;
  font-stretch: 110%;
  font-weight: 400;
  font-size: 18px;
  display: block;
  padding: 20px;
  border: 1px solid #333;
  border-radius: 5px;
  color: white;
  text-decoration: none;
  text-transform: uppercase;
  background-image: url('./assets/pattern-dots-primary.svg');
  border-bottom: 3px solid rgb(255, 136, 0);
  cursor: pointer;

  &.secondary {
    background-image: url('./assets/pattern-dots-secondary.svg');
    border-bottom: 3px solid rgb(0, 183, 255);
    font-size: 17px;
  }

  &.neutral {
    background-image: url('./assets/pattern-dots-neutral.svg');
    border-bottom: 3px solid rgb(0, 255, 13);
    font-size: 17px;
  }

  &:hover {
    border-bottom: 3px solid white;
  }
}

.cta.small {
  font-size: 16px;
  padding: 10px;
  border-bottom-width: 1px;
}

.cta.inactive {
  border-color: black;
  background: #222;
  color: #666;
  background: url('./assets/pattern-dots-disabled.svg');

  svg {
    fill: #666;
  }
}

#showUpload {
  cursor: pointer;
  color: #fff;
  background-color: rgba(0, 0, 0, 0.2);
  padding: 10px;
  border-radius: 5px;
  position: absolute;
  top: 40px;
  right: 20px;
}

footer {
  color: #fff;
  text-align: center;
  padding: 10px;
  bottom: 0;
  width: calc(100vw - 100px);
  height: 75px;
  margin: 0 50px;
  font-size: 11px;
}
</style>
