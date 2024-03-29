<script setup lang="ts">
import _ from 'lodash'

import { ref } from 'vue'

import mapsSynchronizeVue from '../maps/synchronize.vue'
import mapsUploadVue from '../maps/upload.vue'
import mapsOldVue from '../maps/old.vue'

const _showUpload = ref(false)
const showUpload = () => {
  _showUpload.value = true
}
</script>

<template>
  <span id="showUpload" @click="showUpload" v-if="!_showUpload">Upload</span>

  <maps-upload-vue v-if="_showUpload" />
  <maps-old />
  <maps-synchronize-vue />
</template>

<style lang="scss"></style>