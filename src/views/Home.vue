<script setup lang="ts">
import _ from 'lodash'

import { ref } from 'vue'

import MapsSynchronize from '../components/Maps-Synchronize.vue'
import MapsUpload from '../components/Maps-Upload.vue'

const _showUpload = ref(false)
const showUpload = () => {
  _showUpload.value = true
}
</script>

<template>
  <span id="showUpload" @click="showUpload" v-if="!_showUpload">Upload</span>

  <maps-upload v-if="_showUpload" />
  <maps-synchronize />
</template>

<style lang="scss"></style>