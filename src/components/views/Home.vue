<script setup lang="ts">
import _ from 'lodash'

import { ref } from 'vue'

import mapsMainVue from '../maps/main.vue'

import configMainVue from '../config/main.vue'

const _showUpload = ref(false)
const showUpload = () => {
  _showUpload.value = true
}
</script>

<template>
  <maps-upload-vue v-if="_showUpload" />
  <maps-main-vue />
  <config-main-vue />
</template>

<style lang="scss"></style>