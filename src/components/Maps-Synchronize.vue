<script setup lang="ts">
import _ from 'lodash'

import actionsVue from './actions.vue'
import axios from 'axios'


import { invoke } from "@tauri-apps/api/tauri";
import { onMounted, reactive, ref } from 'vue';

import config from '../CONFIG'

const state = ref({
  actions: []
})

const runAction = async (title, executor) => {
  const action = reactive({
    title,
    status: 'pending',
    info: []
  })
  action.status = 'pending'
  state.value.actions.push(action)
  try {
    console.log('starting action', action)
    await executor(action)
    console.log('done with that.', action)
    action.status = 'success'
  } catch (error) {
    action.status = 'error'
    action.info.push(error)
    throw error
  }
}

const synchronize = async () => {
  const CONFIG = await config()
  let localMapData
  await runAction('get local map data', async (action) => {
    localMapData = await invoke("get_map_data");
  })

  let remoteMapData
  await runAction('get remote map data', async (action) => {
    const remote = await axios.get(CONFIG.API_URL + '/maps/hashes')
    remoteMapData = remote.data
  })

  let missingMaps: string[] = []
  let outdatedMaps: string[] = []
  let needAction = false;

  console.log('localMapData', localMapData)
  console.log('remoteMapData', remoteMapData)

  await runAction('check for missing/outdated maps', async (action) => {
    missingMaps = _.difference(Object.keys(remoteMapData), Object.keys(localMapData))
    outdatedMaps = Object.keys(localMapData).filter((key) => {
      if (!remoteMapData[key]) return false
      console.log('checking outdated', key, localMapData[key], remoteMapData[key])
      return localMapData[key] !== remoteMapData[key]
    })

    needAction = missingMaps.length > 0 || outdatedMaps.length > 0

    if (!needAction) {
      action.info.push('all maps up to date.')
    }
  })

  if (!needAction) {
    console.log('NO ACTION NEEDED')
    return
  }

  if (missingMaps.length > 0) {
    console.log('MISSING MAPS', missingMaps)
    await runAction('download missing maps', async (action) => {
      for (const map of missingMaps) {
        action.info.push(`downloading ${map}...`)
        await invoke("download_map", { map })
        console.log('DONE DL MAP')
        action.info[action.info.length - 1] += ` done.`
      }
    })
  }

  if (outdatedMaps.length > 0) {
    console.log('OUTDATED MAPS', outdatedMaps)
    await runAction('update outdated maps', async (action) => {
      for (const map of outdatedMaps) {
        action.info.push(`updating ${map}...`)
        await invoke("download_map", { map })
        console.log('DONE UPD MAP')
        action.info[action.info.length - 1] += ` done.`
      }
    })
  }
  await runAction('update cache file', async action => {
    await invoke("update_map_cache", { data: remoteMapData })
  })

  console.log('DONE WITH ALL')

}
onMounted(async () => {
  try {
    await synchronize()
  } catch (error) {
    console.error(error)
  }
})
</script>

<template>
  <h2>MAPS</h2>
  <actions-vue :actions="state.actions" />
</template>

<style lang="scss">
ul {
  list-style: none;
  padding: 0;
}

span.title {
  font-size: 1.5em;
}
</style>