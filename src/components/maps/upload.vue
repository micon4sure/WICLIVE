<script setup lang="ts">
import _ from 'lodash'
import { reactive, ref } from 'vue';
import axios from 'axios'
import config from '../../get_config'

import jobsVue from '../jobs.vue'
import createJob, { Job } from '../../lib/wic-job';

const _jobs: Job[] = reactive([])

const $file = ref(null)

// check for key in local storage
const _key = ref(null)
if (localStorage.getItem('upload-key')) {
  console.log('upload-key', localStorage.getItem('upload-key'))
  _key.value = localStorage.getItem('upload-key')
}

const upload = async () => {
  const CONFIG: any = await config()
  
  const job = createJob('Upload map', async (addInfo) => {
    if (!_key.value) {
      throw new Error('No API Key')
    }
    if (!$file.value.value) {
      throw new Error('No File')
    }

    const filename = $file.value.files![0].name;

    addInfo(`Uploading ${filename}...`)

    const formData = new FormData()
    formData.append('file', $file.value.files![0])
    formData.append('key', _key.value)

    try {
      await axios.post(CONFIG.API_URL + '/maps/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data'
        }
      })
    } catch (error) {
      console.error(error)
      throw new Error(error + ' (' + error.response?.data + ')')
    }

    addInfo('Upload complete..')

    // set key in local storage
    localStorage.setItem('upload-key', _key.value)
  })

  _jobs.push(job)
  await job.run()
};
</script>

<template>
  <div class="card" id="upload">
    <div class="card-header">
      Upload Map
    </div>
    <div class="card-body">
      <div class="mb-3">
        <input type="file" id="file" class="form-control" ref="$file" />
      </div>
      <div class="mb-3">
        <label for="key" class="form-label">API KEY</label>
        <input type="text" id="key" class="form-control" placeholder="API KEY" v-model="_key">
      </div>
      <button type="button" id="upload" @click="upload" class="btn cta small">Upload</button>
      <jobs-vue :jobs="_jobs" v-if="_jobs.length" />
    </div>
  </div>
</template>

<style lang="scss">
#upload {
  margin-bottom: 10px;
}
</style>