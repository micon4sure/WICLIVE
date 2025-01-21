<script setup lang="ts">
import _ from 'lodash'

import { ref } from 'vue'

import mapsMainVue from '../maps/main.vue'
import configMainVue from '../config/main.vue'
import setupVue from '../setup/main.vue'
</script>

<template>
  <div>
    <setup-vue />
    <maps-upload-vue />
    <maps-main-vue />
    <config-main-vue />
  </div>
</template>

<style lang="scss"></style>