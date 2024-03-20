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
    <h1>WIC LIVE <small>{{ _version }}</small></h1>
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

h1 small {
  font-size: 12px;
  color: #fff;

}

#app {
  display: flex;
  flex-direction: column;
  background: url("./assets/pattern.svg");
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
  width: calc(100vw - 100px);
  max-width: calc(100vw - 100px);
  flex: 1;
}


#showUpload {
  cursor: pointer;
  color: #fff;
  background-color: #000;
  padding: 10px;
  border-radius: 5px;
  margin: 10px;
  position: absolute;
  top: 10px;
  right: 60px;
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
