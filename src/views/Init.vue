<script setup lang="ts">
import _ from 'lodash'

import { onMounted, reactive } from 'vue'
import { invoke } from '@tauri-apps/api';

import jobsVue from '../components/jobs.vue'

const state = reactive({
  jobs: []
})


const runJob = async (title, executor) => {
  const job = reactive({
    title,
    status: 'pending',
    info: [],
    progress: null
  })
  state.jobs.push(job)
  let result = null;
  try {
    result = await executor(job)
    job.status = 'success'
  } catch (error) {
    job.status = 'error'
    job.info.push(error)
  }
  return result
}


onMounted(async () => {
  try {
    let installPath
    await runJob('Find install path', async (job) => {
      job.info.push('Looking for install path...')
      installPath = await invoke('find_install_path')
      job.info.push(`Found install path: ${installPath}`)
    })

    await runJob('Extract game version', async (job) => {
      job.info.push('Extracting game version...')
      const gameVersion = await invoke('extract_game_version', { installPath })
      job.info.push(`Game version: ${gameVersion}`)
    });
  } catch (error) {
    console.error(error)
  }
})

</script>

<template>
  <h2>INIT</h2>
  <jobsVue :jobs="state.jobs" />
</template>

<style lang="scss"></style>