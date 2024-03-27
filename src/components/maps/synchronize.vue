<script setup lang="ts">
import _ from 'lodash'

import jobsVue from '../jobs.vue'
import axios from 'axios'

import { invoke } from "@tauri-apps/api/tauri";
import { computed, onMounted, reactive, ref, watch } from 'vue';

import get_config from '../../get_config'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';
import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';
import iconRefresh from '@fortawesome/fontawesome-free/svgs/solid/rotate.svg';

import WIC_DownloadProgress from '../../lib/wic-download-progress'

import { WIC_Map_Frontend, WIC_Map_Status } from '../../lib/wic-map'

const progress = new WIC_DownloadProgress
let remoteData = {} as any

const state = ref({
  jobs: [],
  maps: [] as WIC_Map_Frontend[]
})

const runJob = async (title, executor) => {
  const job = reactive({
    title,
    status: 'pending',
    info: [],
    progress: null
  })
  state.value.jobs.push(job)
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

const init = async () => {
  const CONFIG: any = await get_config()
  const local: Array<String> = await invoke("get_map_files");
  console.log('local maps are', local)

  const remote = (await axios.get(CONFIG.API_URL + '/maps/data')).data
  remoteData = remote

  let promises = _.map(remote, async (map) => {
    let status: WIC_Map_Status;
    if (!_.includes(local, map.name)) {
      status = WIC_Map_Status.MISSING
    } else {
      const hash = await invoke("get_map_hash", { filename: map.name })
      if (hash != map.hash) {
        status = WIC_Map_Status.OUTDATED
      } else {
        status = WIC_Map_Status.CURRENT
      }
    }

    let size = map.size / 1024 / 1024
    size = Math.round(size * 100) / 100
    const data = {
      name: map.name,
      status: status,
      date: map.date,
      uploader: map.uploader,
      version: map.version,
      size
    } as WIC_Map_Frontend

    state.value.maps.push(data)
  })
  await Promise.all(promises)

}

const queue = []
let busy = false
const downloadMap = async name => {
  if (queue.includes(name)) return;
  queue.push(name)
  _.find(state.value.maps, { name: name }).status = WIC_Map_Status.PENDING
  if (busy) return;
  busy = true

  while (queue.length) {
    const name = queue.shift()

    await runJob(`downloading ${name}`, async (job) => {
      const progressKey = progress.on(name, (event) => {
        job.progress = event.percentage
      })

      const map = _.find(state.value.maps, { name: name })
      map.status = WIC_Map_Status.LOADING

      await invoke("download_map", { map: name })

      job.info.push('building hash...')
      const hash: string = await invoke("get_map_hash", { filename: name })

      if (remoteData[name].hash != hash) {
        map.status = WIC_Map_Status.OUTDATED
        console.log('hash mismatch', remoteData[name].hash, hash)
        throw new Error('hash mismatch')
      }

      map.hash = hash
      progress.off(progressKey)
      job.info.push('done.')
      map.status = WIC_Map_Status.CURRENT
    })
  }
  busy = false
}

// watch for action needed
const actionNeeded = ref(false)
watch(() => state.value.maps, () => {
  actionNeeded.value = _.some(state.value.maps, (map) => map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED)
}, { deep: true })

// computed sorted maps
const _maps = computed(() => {
  return _.orderBy(state.value.maps, [
    (map) => {
      if (map.status == WIC_Map_Status.MISSING) return 0;
      if (map.status == WIC_Map_Status.OUTDATED) return 1;
      if (map.status == WIC_Map_Status.LOADING) return 2;
      if (map.status == WIC_Map_Status.PENDING) return 3;
      return 4
    },
    map => map.name
  ])
})

const synchronize = async () => {
  if (!actionNeeded.value) return;
  runJob('synchronizing', async (job) => {
    const promises = _.map(state.value.maps, async (map) => {
      if (map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED) {
        await downloadMap(map.name)
      }
    })
    await Promise.all(promises)
  })
}

onMounted(async () => {
  await runJob('initializing', async (job) => {
    await init()
  })
})
</script>

<template>
  <h2>MAPS</h2>
  <div id="maps">
    <div id="maps-list-container" :class="{ hidden: !state.maps.length }">
      <div id="maps-list-actions">
        <span class="btn-container secondary">
        </span>
        <span class="btn-container primary" @click="synchronize" :class="{ inactive: !actionNeeded }">
          <button class="btn">
            <iconDownload class="icon" />
          </button>
          Download all missing/outdated
        </span>
      </div>
      <table id="maps-list" v-if="_maps.length">
        <tr v-for="map in _maps" :key="map.name">
          <th>
            {{ map.name }}
          </th>
          <td>
            v{{ map.version }}
          </td>
          <td>
            {{ map.uploader }}
          </td>
          <td>
            {{ map.date }}
          </td>
          <td>
            {{ map.size }} MB
          </td>
          <td>
            <span v-if="map.status != WIC_Map_Status.CURRENT">{{ map.status }}</span>
          </td>
          <td class="status">
            <span class="btn-container primary" @click="downloadMap(map.name.toString())"
              v-if="map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED">
              <button class="btn btn-sm btn-secondary">
                <iconDownload class="icon" />
              </button>
              Download
            </span>
            <div class="spinner-border" role="status" v-if="map.status == WIC_Map_Status.LOADING">
              <span class="sr-only">&nbsp;</span>
            </div>
            <iconCheck class="icon map-current" v-if="map.status == WIC_Map_Status.CURRENT" />
          </td>
        </tr>
      </table>
    </div>
    <jobs-vue :jobs="state.jobs" id="maps-jobs" />
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
    flex: 1;
    cursor: pointer;
    display: inline-block;
    justify-content: space-between;
    align-items: center;
    height: 35px;
    border: none;
    border-radius: 5px;
    // background: linear-gradient(0deg, #791c05 0%, #ce2e06 100%);
    height: 35px;
    line-height: 35px;
    padding: 0 10px;
    text-align: left;
    text-wrap: nowrap;

    &.primary {
      background-image: url('../assets/pattern-dots-primary.svg');
    }

    &.secondary {
      background-image: url('../assets/pattern-dots-secondary.svg');

      color: #aaa;

      svg {
        fill: #aaa;
      }
    }

    button {
      height: 35px;
      line-height: 15px;
      border: none;
      background: transparent;
    }
  }

  #maps-list-container {
    &.hidden {
      visibility: hidden;
    }

    width: 65%;
    padding: 0;
    margin: 0;
    border: 1px solid #333;
    border-top-right-radius: 5px;
    border-top-left-radius: 10px;
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 5px;
    background: rgba(255, 255, 255, .1);

    button {
      margin: 0;
    }

  }

  #maps-list-actions {

    height: 50px;

    display: flex;
    justify-content: flex-end;
    background: #333;

    border-top-left-radius: 5px;
    border-top-right-radius: 5px;

    .btn-container {
      height: 50px;
      padding: 0 15px;
      height: 50px;
      line-height: 50px;
      border-radius: 0;

      &:first-of-type {
        border-top-left-radius: 10px;
      }

      &:last-of-type {
        border-top-right-radius: 5px;
      }

      &.inactive {
        background: #222;
        color: #666;

        svg {
          fill: #666;
        }
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
    background: linear-gradient(to right, rgba(255, 255, 255, .2), rgba(255, 255, 255, .05));
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 5px;

    .btn-container {
      font-size: 14px;
    }

    tr {
      border-bottom: 1px solid #444;

      &:last-of-type {
        border-bottom: none;
      }
    }

    td,
    th {
      padding: 10px;
    }

    td {
      font-size: 11px;
    }

    td .spinner-border {
      color: rgb(0, 162, 255);
    }

    td.status {
      text-align: right;
    }

    .icon.map-current {
      fill: #15a315;
      height: 1.5em;

    }
  }

  #maps-jobs {
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