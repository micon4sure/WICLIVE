<script setup lang="ts">
import _ from 'lodash'
import { reactive, ref } from 'vue';
import jobsVue from './jobs.vue'
import axios from 'axios'
import config from '../get_config'

const _uploadJobs = ref([])
const $file = ref(null)

// check for key in local storage
const _key = ref(null)
if (localStorage.getItem('upload-key')) {
  console.log('upload-key', localStorage.getItem('upload-key'))
  _key.value = localStorage.getItem('upload-key')
}

const upload = async () => {
  const CONFIG: any = await config()
  const uploadJob = reactive({
    title: 'upload map',
    status: 'pending',
    info: []
  })
  _uploadJobs.value.push(uploadJob)

  if (!_key.value) {
    uploadJob.status = 'error'
    uploadJob.info.push('No API Key')
    return
  }
  if (!$file.value.value) {
    uploadJob.status = 'error'
    uploadJob.info.push('No File')
    return
  }

  const filename = $file.value.files![0].name;

  uploadJob.info.push(`Uploading ${filename}...`)

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
    uploadJob.status = 'error'
    console.error(error)
    uploadJob.info.push(error + ' (' + error.response?.data + ')')
    return
  }

  uploadJob.info.push('Upload complete..')
  uploadJob.status = 'success'

  // set key in local storage
  localStorage.setItem('upload-key', _key.value)
}
</script>

<template>
  <div>
    <input type="file" id="file" ref="$file" class="form-control" />
    <input type="text" id="key" placeholder="API KEY" class="form-control" v-model="_key" />
    <button id="upload" @click="upload" class="btn btn-primary">Upload</button>
    <jobs-vue :jobs="_uploadJobs" />
  </div>
</template>

<style lang="scss"></style>