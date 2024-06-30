<script setup lang="ts">
import _ from 'lodash'

import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api';

import wicautoexecMinimumContents from '../../config_txt/wicautoexec_minimum.txt?raw'
import proBindings from '../../config_txt/pro_bindings.txt?raw'

const fnWicautoexec = 'wicautoexec.txt'
const fnControllerOptions = 'Controller Options.txt'

const initializeConfig = async () => {
  // create autoexec if not exists
  let autoexecExists = await invoke('file_exists', { path: fnWicautoexec })
  if (!autoexecExists) {
    await invoke('set_file_contents', { path: fnWicautoexec, contents: wicautoexecMinimumContents })
    await invoke('set_file_contents', { path: fnWicautoexec + '.bak', contents: wicautoexecMinimumContents })
    return;
  }

  // create backup if not exists
  const autoexecBakExists = await invoke('file_exists', { path: fnWicautoexec + '.bak' })
  if (!autoexecBakExists) {
    const autoexecContents = await invoke('get_file_contents', { path: fnWicautoexec });
    await invoke('set_file_contents', { path: fnWicautoexec + '.bak', contents: autoexecContents })
  }
}


// # LIVE
const _liveEnabled = ref(false)
const liveEnabled = async () => {
  try {
    const contents: string = await invoke('get_file_contents', { path: fnWicautoexec })
    // check if first line is  "// LIVE"
    return contents.split('\r\n')[0].trim() === '// LIVE';
  } catch (error) {
    return false;
  }
}
const enableLive = async () => {
  initializeConfig();
  // set live contents if exists, otherwise set minimum contents
  let liveAutoexecExists = await invoke('file_exists', { path: fnWicautoexec + '.live' })
  let contents = liveAutoexecExists
    ? await invoke('get_file_contents', { path: fnWicautoexec + '.live' })
    : wicautoexecMinimumContents
  await invoke('set_file_contents', { path: fnWicautoexec, contents })

  // remove live file if exists
  if (liveAutoexecExists) {
    await invoke('remove_file', { path: fnWicautoexec + '.live' })
  }
  _liveEnabled.value = true
}

// # COMPETITIVE
const _competitiveEnabled = ref(false)
const competitiveEnabled = async () => {
  if (!await invoke('file_exists', { path: fnWicautoexec }))
    return false

  try {
    const contents: string = await invoke('get_file_contents', { path: fnWicautoexec })

    // check for line "// competitive on|off"
    const competitiveLine = contents.split('\r\n').find(line => {
      return line.trim().startsWith('// competitive')
    })
    if (!competitiveLine)
      return false

    return competitiveLine.trim().split(' ')[2] === 'on'
  } catch (error) {
    console.error(error)
    return false;
  }
}

const enableCompetitve = async () => {
  initializeConfig();
  const contents: string = await invoke('get_file_contents', { path: fnWicautoexec })

  // change line "// competitive off"
  const lines = contents.split('\r\n')
  const newLines = lines.map(line => {
    if (line.trim().startsWith('// competitive off')) {
      const [_, __, state] = line.split(' ')
      return `// competitive on`
    }
    return line
  })
  newLines.push('SetFogDistances 1 1 1 1')
  newLines.push('Ex3DRenderClouds 0')

  await invoke('set_file_contents', { path: fnWicautoexec, contents: newLines.join('\r\n') })

  _competitiveEnabled.value = true
}
const disableCompetitve = async () => {
  initializeConfig();
  const contents: string = await invoke('get_file_contents', { path: fnWicautoexec })

  // change line "// competitive off"
  const lines = contents.split('\r\n')
  const newLines = lines.map(line => {
    if (line.trim().startsWith('// competitive on')) {
      const [_, __, state] = line.split(' ')
      return `// competitive off`
    }
    return line
  })
  // remove lines
  _.remove(newLines, line => line === 'SetFogDistances 1 1 1 1')
  _.remove(newLines, line => line === 'Ex3DRenderClouds 0')

  await invoke('set_file_contents', { path: fnWicautoexec, contents: newLines.join('\r\n') })

  _competitiveEnabled.value = false
}
const restoreSettings = async () => {
  const currentContents: string = await invoke('get_file_contents', { path: fnWicautoexec })
  const bakContents: string = await invoke('get_file_contents', { path: fnWicautoexec + '.bak' })
  await invoke('set_file_contents', { path: fnWicautoexec + '.live', contents: currentContents })
  await invoke('set_file_contents', { path: fnWicautoexec, contents: bakContents })
  await invoke('remove_file', { path: fnWicautoexec + '.bak' })
  _liveEnabled.value = false
  _competitiveEnabled.value = false
}

// # PRO KEYBINDINGS
const _proKeybindingsEnabled = ref(false)
const proKeybindingsEnabled = async () => {
  try {
    const contents: string = await invoke('get_file_contents', { path: fnControllerOptions })
    // check if first line is  "// LIVE PRO"
    return contents.split('\n')[0].trim() === '// LIVE PRO';
  } catch (error) {
    return false;
  }
}
const enableProKeybindings = async () => {
  const contents: string = await invoke('get_file_contents', { path: fnControllerOptions })
  const controllerOptionsBakExists = await invoke('file_exists', { path: fnControllerOptions + '.bak' })
  if (!controllerOptionsBakExists) {
    await invoke('set_file_contents', { path: fnControllerOptions + '.bak', contents })
  }

  await invoke('set_file_contents', { path: fnControllerOptions, contents: proBindings })
  _proKeybindingsEnabled.value = true
}
const restoreKeybindings = async () => {
  const contents: string = await invoke('get_file_contents', { path: fnControllerOptions + '.bak' })
  await invoke('set_file_contents', { path: fnControllerOptions, contents })
  // remove bak
  await invoke('remove_file', { path: fnControllerOptions + '.bak' })
  _proKeybindingsEnabled.value = false
}

onMounted(async () => {
  _competitiveEnabled.value = await competitiveEnabled()
  _liveEnabled.value = await liveEnabled()
  _proKeybindingsEnabled.value = await proKeybindingsEnabled()
})
</script>

<template>
  <div id="config">
    <div class="card">
      <div class="card-header">wicautoexec</div>
      <div class="card-body">
        <p>
          Enabling live settings will enable hotkeys for TA usage. Competitive settings remove fog and clouds.
        </p>
        <ul>
          <li>F1 = Aerial Recon</li>
          <li>F2 = Air2Air</li>
          <li>F3 = Tankbuster</li>
          <li>F4 = Larty</li>
          <li>F5 = Harty</li>
          <li>7 = Jeepdrops</li>
          <li>8 = Tankdrops</li>
          <li>9 = Airbornes</li>
          <li>0 = Cluster</li>
        </ul>
        <div class="wicautoexec-buttons">
          <button class="cta small" @click="enableLive" v-if="!_liveEnabled">Enable LIVE settings</button>
          <button v-else class="cta small secondary" @click="restoreSettings">Restore wicautoexec</button>
          <div v-if="_liveEnabled">
            <button class="cta small neutral" @click="enableCompetitve" v-if="!_competitiveEnabled">Enable Competitive
              settings</button>
            <button class="cta small neutral" @click="disableCompetitve" v-else>Disable Competitive settings</button>
          </div>
        </div>
      </div>
    </div>
    <!-- <div class="card">
      <div class="card-header">Controller Options</div>
      <div class="card-body">
        <p>Pro Keybindings are</p>
        <ul>
          <li>Q = Offensive ability</li>
          <li>E = Defensive ability</li>
          <li>R = Reverse move</li>
          <li>T = Hold fire</li>
          <li>Y = Stop</li>
          <li>X = Unload</li>
          <li>C = Force move</li>
          <li>V = Enter nearest transport</li>
        </ul>
        <button class="cta small" @click="enableProKeybindings" v-if="!_proKeybindingsEnabled">Enable Pro Key
          Bindings</button>
        <button class="cta small" @click="restoreKeybindings" v-else>Restore
          Key Bindings</button>
      </div>
    </div> -->
  </div>
</template>

<style lang="scss">
#config {
  margin-top: 25px;

  .card:first-of-type {
    margin-bottom: 15px;
  }
}

.wicautoexec-buttons {
  margin-top: 15px;
  display: flex;
  flex-wrap: wrap;
  gap: 1rem;
}
</style>