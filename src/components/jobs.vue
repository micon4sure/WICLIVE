<script setup lang="ts">
import jobVue from './job.vue'
// actions as prop
import { computed, defineProps } from 'vue'
import type { Job } from '../lib/wic-job'

defineProps<{
  jobs: Job[]
}>()

</script>

<template>
  <ul class="jobs">
    <li v-for="(job, idx) in jobs" :key="idx + job.data.title" :class="['action', job.data.status]">
      <job-vue :job="job" />
    </li>
  </ul>
</template>

<style lang="scss">
.jobs {
  padding: 10px;

  width: 35%;
  background: rgba(0, 0, 0, .4);
  padding: 10px;
  border-radius: 10px;
  border-top-left-radius: 5px;
  border-bottom-left-radius: 5px;

  ul,
  li {
    list-style: none;
    padding: 0;
    margin: 0;
  }
}
</style>
