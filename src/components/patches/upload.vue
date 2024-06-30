<script setup lang="ts">
import _ from 'lodash'
import { reactive, ref } from 'vue';
import jobsVue from '../jobs.vue'
import axios from 'axios'
import config from '../../get_config'

import wicJobs from '../../lib/wic-jobs';

const manager = wicJobs.patchManager
const progress = wicJobs.progress
const _jobs = manager.getJobs()


const $file = ref(null)

// check for key in local storage
const _key = ref(null)
if (localStorage.getItem('upload-key')) {
  console.log('upload-key', localStorage.getItem('upload-key'))
  _key.value = localStorage.getItem('upload-key')
}

const upload = async () => {
  const CONFIG: any = await config()
  manager.runJob('Upload patch', async (job) => {
    if (!_key.value) {
      job.status = 'error'
      job.info.push('No API Key')
      return
    }
    if (!$file.value.value) {
      job.status = 'error'
      job.info.push('No File')
      return
    }

    const filename = $file.value.files![0].name;
    console.log($file.value.files![0], filename)

    job.info.push(`Uploading ${filename}...`)

    const formData = new FormData()
    formData.append('file', $file.value.files![0])
    formData.append('key', _key.value)

    try {
      await axios.post(CONFIG.API_URL + '/patches/upload', formData, {
        headers: {
          'Content-Type': 'multipart/form-data'
        }
      })
    } catch (error) {
      job.status = 'error'
      console.error(error)
      job.info.push(error + ' (' + error.response?.data + ')')
      return
    }

    job.info.push('Upload complete..')
    job.status = 'success'

    // set key in local storage
    localStorage.setItem('upload-key', _key.value)
  })
};
</script>


<template>
  <div class="card" id="upload-patch">
    <div class="card-header">
      Upload Patch
    </div>
    <div class="card-body">
      <div class="mb-3">
        <input type="file" id="file" class="form-control" ref="$file" />
      </div>
      <div class="mb-3">
        <label for="key" class="form-label">API KEY</label>
        <input type="text" id="key" class="form-control" placeholder="API KEY" v-model="_key">
      </div>
      <button type="button" @click="upload" class="cta small">Upload</button>
    </div>
  </div>
</template>

<style lang="scss"></style>