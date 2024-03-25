<script setup lang="ts">
import _ from 'lodash'
import jobVue from './job.vue'
// actions as prop
import { defineProps, ref, onMounted, watch } from 'vue'
const props = defineProps({
  jobs: Array
})
let jobs = props.jobs as Array<any>

const jobsSuccess = ref([]);
const jobsError = ref([]);
const jobsPending = ref([]);

watch(jobs, (newVal) => {
  jobsSuccess.value = jobs.filter((job) => job.status === 'success');
  jobsError.value = jobs.filter((job) => job.status === 'error');
  jobsPending.value = jobs.filter((job) => job.status === 'pending');
})
</script>

<template>
  <ul>
    <li v-for="(action, idx) in  jobs " :key="idx + action.title" :class="['action', action.status]">
      <job-vue :action="action" />
    </li>
  </ul>
</template>

<style lang="scss"></style>