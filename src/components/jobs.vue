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
  <ul class="jobs">
    <li v-for="(job, idx) in jobs " :key="idx + job.title" :class="['action', job.status]">
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