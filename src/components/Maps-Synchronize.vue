<script setup lang="ts">
import _ from 'lodash'

import actionsVue from './actions.vue'
import axios from 'axios'
import WIC_Cache, { WIC_Map } from '../wic-cache'


import { invoke } from "@tauri-apps/api/tauri";
import { onMounted, reactive, ref, watch } from 'vue';

import get_config from '../get_config'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';
import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';

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
    await executor(action)
    action.status = 'success'
  } catch (error) {
    action.status = 'error'
    action.info.push(error)
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
  await runAction('complete local cache', async (action) => {
    let newHash = false;
    console.log(cache)
    for (const filename of intersection) {
      let hash;
      if (cache.has(filename) && cache.get(filename).hash) {
        hash = cache.get(filename).hash
      } else {
        action.info.push('hashing ' + filename)
        hash = await invoke("get_map_hash", { filename })
        newHash = true;
      }

      const map = { name: filename, hash } as WIC_Map;
      cache.set(filename, map)

      const index = maps.value.findIndex((map) => map.name === filename)
      maps.value[index].status = hash == remoteMapData[filename] ? 'current' : 'outdated'
    }
    if (newHash) {
      action.info.push('done.')
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
    listEntry.status = 'pending'
    await invoke("download_map", { map: filename })
    action.info.push('checking hash')
    console.log('building hash')
    const hash: string = await invoke("get_map_hash", { filename })
    console.log('building hash done', hash)
    let success = remoteMapData[filename] == hash
    if (!success) {
      listEntry.status = 'outdated'
      console.log('hash mismatch', remoteMapData[filename], hash)
      throw new Error('hash mismatch')
    }
    action.info.push('done.')
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
      <div id="maps-list-synchronize" :class="{ inactive: !actionNeeded }" @click="synchronize">
        <span class="btn-container">
          <button class="btn btn-secondary">
            <iconDownload class="icon" />
          </button>
          Download all missing/outdated
        </span>
      </div>
      <table id="maps-list">
        <tr v-for="map in maps" :key="map.name.toString()">
          <td>{{ map.name }}</td>
          <td class="status">
            <span class="btn-container" @click="downloadMap(map.name.toString())"
              v-if="map.status == 'missing' || map.status == 'outdated'">
              <button class="btn btn-sm btn-secondary">
                <iconDownload class="icon" />
              </button>
              Download
            </span>
            <div class="spinner-border" role="status" v-if="map.status == 'pending'">
              <span class="sr-only">&nbsp;</span>
            </div>
            <iconCheck class="icon map-current" v-if="map.status == 'current'" />

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

  .spacer {
    flex: 1;
  }

  .btn-container {
    cursor: pointer;
    display: inline-block;
    justify-content: space-between;
    align-items: center;
    height: 35px;
    border: none;
    border-radius: 5px;
    background-image: url('../assets/pattern-dots.svg');
    // background: linear-gradient(0deg, #791c05 0%, #ce2e06 100%);
    height: 35px;
    line-height: 35px;
    padding: 0 10px;
    text-align: left;

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

    display: flex;
    justify-content: flex-end;

    .btn-container {
      height: 50px;
      padding: 0 15px;
      height: 50px;
      line-height: 50px;
      border-radius: 0;
      border-top-right-radius: 5px;
    }

    &.inactive span {
      background: #222;
    }

    &.inactive {
      background: #333;
      color: #666;

      svg {
        fill: #666;
      }
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
      color: rgb(0, 162, 255);
    }

    td.status {
      text-align: right;
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

    .icon.map-current {
      fill: #15a315;
      height: 1.5em;

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