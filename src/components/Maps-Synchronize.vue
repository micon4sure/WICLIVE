<script setup lang="ts">
import _ from 'lodash'

import actionsVue from './actions.vue'
import axios from 'axios'
import WIC_Cache, { WIC_Map } from '../wic-cache'


import { invoke } from "@tauri-apps/api/tauri";
import { onMounted, reactive, ref, watch } from 'vue';

import get_config from '../get_config'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';

const state = ref({
  actions: [],
  missingMaps: [],
  outdatedMaps: []
})

const runAction = async (title, executor) => {
  const action = reactive({
    title,
    status: 'pending',
    info: []
  })
  state.value.actions.push(action)
  try {
    // console.log('starting action', action)
    await executor(action)
    // console.log('done with that.', action)
    action.status = 'success'
  } catch (error) {
    action.status = 'error'
    action.info.push(error)
    throw error
  }
}

let maps = ref([] as { name: String, status: String }[])
const cache = WIC_Cache.instance();
let remoteMapData

const initialize = async () => {
  while (state.value.actions.length > 0) {
    state.value.actions.pop()
  }
  const CONFIG: any = await get_config()

  // read local map files
  let localMapFiles
  await runAction('read local map files', async (action) => {
    localMapFiles = await invoke("get_map_files");
    console.log('localMapData', localMapFiles)
    localMapFiles = localMapFiles;
  })

  // read remote map data
  await runAction('get remote map data', async (action) => {
    const remote = await axios.get(CONFIG.API_URL + '/maps/hashes')
    remoteMapData = remote.data
  })

  // get intersection of local and remote maps
  const intersection = _.intersection(Object.keys(remoteMapData), localMapFiles)
  console.log({ remote: Object.keys(remoteMapData), localMapFiles, intersection })

  // create map list
  _.each(intersection, (filename) => {
    maps.value.push({ name: filename, status: "?" })
  });


  // get hashes where needed
  await runAction('complete local cache', async (actions) => {
    for (const filename of intersection) {
      let hash;
      if (cache.has(filename)) {
        hash = cache.get(filename).hash
      } else {
        const hash = await invoke("get_map_hash", { filename })
      }

      actions.info.push('hashing ' + filename)
      console.log('hash', filename, hash)
      const map = { name: filename, hash } as WIC_Map;
      cache.set(filename, map)

      const index = maps.value.findIndex((map) => map.name === filename)
      maps.value[index].status = hash == remoteMapData[filename] ? 'current' : 'outdated'
    }
  })

  const missingMaps = _.difference(Object.keys(remoteMapData), localMapFiles)
  console.log('missingMaps', missingMaps)
  _.each(missingMaps, (filename) => {
    maps.value.push({ name: filename, status: "missing" })
  })

}
onMounted(async () => {
  try {
    await initialize()
  } catch (error) {
    console.error(error)
  }
})

const downloadMap = async (filename: string) => {
  await runAction(`download map ${filename}`, async (action) => {
    const listEntry = _.find(maps.value, { name: filename });
    listEntry.status = 'downloading'
    await invoke("download_map", { map: filename })
    listEntry.status = 'building hash'
    console.log('building hash')
    const hash: string = await invoke("get_map_hash", { filename })
    console.log('building hash done', hash)
    let current = remoteMapData[filename] == hash
    if (!current) {
      action.status = 'error'
      action.info.push('hash mismatch')
      return;
    }
    listEntry.status = 'current'
    cache.set(filename, { name: filename, hash })
  })
}

const synchronize = async () => {
  if (!actionNeeded.value) return;
  await runAction('synchronizing', async (action) => {
    for (const map of maps.value) {
      if (map.status == 'missing' || map.status == 'outdated') {
        await downloadMap(map.name as string)
      }
    }
  })
}

const actionNeeded = ref(false)
watch(maps.value, () => {
  actionNeeded.value = maps.value.some((map) => map.status == 'missing' || map.status == 'outdated')
})
</script>

<template>
  <h2>MAPS</h2>
  <div id="maps">
    <div id="maps-list-container">
      <span id="maps-list-synchronize" :class="{ 'btn-container': true, inactive: !actionNeeded }" @click="synchronize">
        <button class="btn btn-secondary">
          <iconDownload class="icon" />
        </button>
        Download missing/outdated
      </span>
      <table id="maps-list">
        <tr>
          <th>Map</th>
          <th>Status</th>
          <th>Actions</th>
        </tr>
        <tr v-for="map in maps" :key="map.name.toString()">
          <td>{{ map.name }}</td>
          <td>{{ map.status }}</td>
          <td>
            <span class="btn-container" @click="downloadMap(map.name.toString())"
              v-if="map.status == 'missing' || map.status == 'outdated'">
              <button class="btn btn-sm btn-secondary">
                <iconDownload class="icon" />
              </button>
              Download
            </span>
            <div class="spinner-border text-primary" role="status" v-if="map.status == 'downloading'">
              <span class="sr-only">&nbsp;</span>
            </div>
          </td>
        </tr>
      </table>
    </div>
    <actions-vue :actions="state.actions" id="maps-actions" />
  </div>
</template>

<style lang="scss">
#maps {
  .btn {
    margin: 5px 0;
  }

  display: flex;

  .btn-container {
    cursor: pointer;
    display: block;
    justify-content: space-between;
    align-items: center;
    height: 35px;
    border: none;
    border-radius: 5px;
    background-image: url('../assets/pattern-dots.svg');
    // background: linear-gradient(0deg, #791c05 0%, #ce2e06 100%);
    height: 35px;
    line-height: 35px;
    padding: 0 0 0 10px;

    button {
      height: 35px;
      line-height: 15px;
      border: none;
      // border-radius: 5px;
      // border-top-left-radius: 0px;
      // border-bottom-left-radius: 0px;
      // background: linear-gradient(0deg, #791c05 0%, #ce2e06 100%);
      background: transparent;
    }
  }

  #maps-list-container {
    padding: 0;
    margin: 0;
    border: 1px solid #333;
    border-top-right-radius: 5px;
    border-top-left-radius: 10px;
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 5px;

    button {
      margin: 0;
    }

    width: 65%;
  }

  #maps-list-synchronize {
    height: 50px;

    &.btn-container {
      border-radius: 5px;
      height: 50px;
      padding: 0 15px;
      height: 50px;
      line-height: 50px;
      border-bottom-left-radius: 0px;
      border-bottom-right-radius: 0px;
    }

    &.inactive span {
      background: #222;
    }

    &.inactive button {
      background: #000;
    }

    button {
      height: 50px;
      line-height: 25px;
      border-radius: 5px;
      padding: 0px;
    }

    .icon {
      margin: 7px 5px;
    }
  }

  #maps-list {
    width: 100%;
    background: linear-gradient(to left, rgba(255, 255, 255, .3), rgba(255, 255, 255, .1));
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 5px;

    td .spinner-border {
      color: rgb(0, 162, 255) !important;
    }

    td,
    th {
      padding: 10px;
    }

    th {
      border-bottom: 1px solid #333;
    }


    tr {
      border-bottom: 1px solid #555;

      &:last-of-type {
        border-bottom: none;
      }
    }
  }

  #maps-actions {
    width: 35%;
    background: rgba(0, 0, 0, .4);
    padding: 10px;
    border-radius: 10px;
    border-top-left-radius: 5px;
    border-bottom-left-radius: 5px;
  }
}

.icon {
  fill: white;
  height: 1em;
  padding-bottom: 3px;
}

#missing-outdated {
  margin-left: 20px;

  h3 {
    margin-top: 15px;
  }
}

ul {
  list-style: none;
  padding: 0;
}

span.title {
  font-size: 1.2em;
}
</style>