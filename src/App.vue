<script setup lang="ts">
import { reactive, ref } from 'vue'

import MapsSynchronize from './components/Maps-Synchronize.vue'
import MapsUpload from './components/Maps-Upload.vue'

const _showUpload = ref(false)
const showUpload = () => {
  _showUpload.value = true
}

</script>

<template>
  <div class="container">
    <h1>WIC LIVE <small>0.0.5</small></h1>
    <span id="showUpload" @click="showUpload" v-if="!_showUpload">Upload</span>

    <maps-upload v-if="_showUpload" />
    <maps-synchronize />
  </div>
</template>

<style scoped>
#showUpload {
  cursor: pointer;
  color: #fff;
  background-color: #000;
  padding: 10px;
  border-radius: 5px;
  margin: 10px;
  position: absolute;
  top: 10px;
  right: 10px;
}
</style>
