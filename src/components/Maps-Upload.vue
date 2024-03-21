<script setup lang="ts">
import _ from 'lodash'
import { reactive, ref } from 'vue';
import actionsVue from './actions.vue'
import axios from 'axios'
import config from '../get_config'

const _uploadActions = ref([])
const $file = ref(null)

// check for key in local storage
const $key = ref(null)
if (localStorage.getItem('upload-key')) {
  $key.value = localStorage.getItem('upload-key')
}

const upload = async () => {
  const CONFIG: any = await config()
  const uploadAction = reactive({
    title: 'upload map',
    status: 'pending',
    info: []
  })
  _uploadActions.value.push(uploadAction)

  if (!$key.value.value) {
    uploadAction.status = 'error'
    uploadAction.info.push('No API Key')
    return
  }
  if (!$file.value.value) {
    uploadAction.status = 'error'
    uploadAction.info.push('No File')
    return
  }

  uploadAction.info.push(`Uploading ${$file.value.files![0].name}...`)

  const formData = new FormData()
  formData.append('file', $file.value.files![0])
  formData.append('key', $key.value.value)

  try {
    await axios.post(CONFIG.API_URL + '/maps/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
  } catch (error) {
    uploadAction.status = 'error'
    console.error(error)
    uploadAction.info.push(error + ' (' + error.response?.data + ')')
  }

  uploadAction.info.push('Upload complete..')
  uploadAction.status = 'success'

  // set key in local storage
  localStorage.setItem('upload-key', $key.value.value)
}
</script>

<template>
  <div>
    <input type="file" id="file" ref="$file" class="form-control" />
    <input type="text" id="key" placeholder="API KEY" ref="$key" class="form-control" />
    <button id="upload" @click="upload" class="btn btn-primary">Upload</button>
    <actions-vue :actions="_uploadActions" />
  </div>
</template>

<style lang="scss"></style>