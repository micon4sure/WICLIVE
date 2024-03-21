<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'

import MapsSynchronize from './components/Maps-Synchronize.vue'
import MapsUpload from './components/Maps-Upload.vue'
import { invoke } from '@tauri-apps/api';

const _showUpload = ref(false)
const showUpload = () => {
  _showUpload.value = true
}

const _version = ref('');
onMounted(async () => {
  const config: any = await invoke('get_config')
  _version.value = config.VERSION
})

</script>

<template>
  <div class="container">
    <h1><img src="./assets/wiclive.png" alt="WIC LIVE" /> <small>{{ _version }}</small></h1>
    <span id="showUpload" @click="showUpload" v-if="!_showUpload">Upload</span>

    <maps-upload v-if="_showUpload" />
    <maps-synchronize />
  </div>
  <footer>
    © 2024 WIC LIVE is NOT affiliated with Ubisoft or Ubisoft Massive.<br />
    © 2009 Ubisoft Entertainment. All Rights Reserved. World in Conflict, Ubisoft Massive the Ubisoft Massive logo,
    Ubisoft, Ubi.com and the Ubisoft logo are all trademarks of Ubisoft Entertainment in the US and/or other countries.
  </footer>
</template>

<style lang="scss">
@font-face {
  font-family: "eurostib";
  src: url("./assets/eurostib.ttf");
}

h1 {
  display: flex;
  align-items: flex-end;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin: 0 -40px 30px -40px;
  padding: 20px 40px;
}


h1 small {
  margin-left: 10px;
  font-size: 12px;
  color: #fff;
}

h2 {
  background: linear-gradient(to right, #055479 0%, transparentize(#ce2e06, 1) 50%);
  border-top-left-radius: 5px;
  border-bottom-left-radius: 5px;
  padding-left: 15px;
}

#app {
  display: flex;
  flex-direction: column;
  background: url("./assets/pattern-stripes.svg");
}

* {
  font-family: "eurostib";
}

body {
  background: linear-gradient(180deg, #082224 0%, #0d2f3f 100%);
  min-height: 100vh;
}

.container {
  padding-top: 20px;
  min-height: calc(100vh - 75px);
  width: calc(100vw - 40px);
  max-width: calc(100vw - 40px);
  flex: 1;
}


#showUpload {
  cursor: pointer;
  color: #fff;
  background-color: rgba(0, 0, 0, 0.2);
  padding: 10px;
  border-radius: 5px;
  position: absolute;
  top: 10px;
  right: 40px;
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
