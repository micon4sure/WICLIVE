<script setup lang="ts">
import _ from 'lodash'

import jobsVue from '../jobs.vue'
import axios from 'axios'

import { invoke } from "@tauri-apps/api/tauri";
import { computed, onMounted, reactive, ref, watch } from 'vue';

import get_config from '../../get_config'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';
import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';

import mapsUploadVue from './upload.vue'

import { WIC_Map_Frontend, WIC_Map_Status } from '../../lib/wic-map'

import wicJobs from '../../lib/wic-jobs';

const manager = wicJobs.manager
const progress = wicJobs.progress
const _jobs = wicJobs._jobs
manager.clearJobs();

let remoteData = {} as any

const state = ref({
  mapsCustom: [],
  mapsLive: [] as WIC_Map_Frontend[]
})

const init = async () => {
  const CONFIG: any = await get_config()
  const local: Array<String> = await invoke("get_map_files");

  // init custom maps
  let custom = [
    'do_Airport.sdf',
    'tw_Arizona.sdf',
    'tw_bocage.sdf',
    'as_Ozzault.sdf',
    'do_Paradise.sdf',
    'do_Valley.sdf',
    'virginia.sdf'
  ]
  const diff = _.difference(custom, local)
  console.log({ custom, local, diff })
  state.value.mapsCustom = _.map(diff, (map) => {
    return reactive({
      name: map,
      status: WIC_Map_Status.MISSING
    })
  })

  // init LIVE maps
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

    state.value.mapsLive.push(data)
  })
  await Promise.all(promises)

}

const queueLive = []
let busyLive = false
const downloadLiveMap = async name => {
  if (queueLive.includes(name)) return;
  queueLive.push(name)
  _.find(state.value.mapsLive, { name: name }).status = WIC_Map_Status.PENDING
  if (busyLive) return;
  busyLive = true

  while (queueLive.length) {
    const name = queueLive.shift()

    await manager.runJob(`Download ${name}`, async (job) => {
      const progressId = progress.on({ type: 'download-map-live' }, (progress) => {
        job.progress = progress.percentage
      })

      const map = _.find(state.value.mapsLive, { name: name })
      map.status = WIC_Map_Status.LOADING

      await invoke("download_map_live", { map: name })

      job.info.push('Compute hash...')
      const hash: string = await invoke("get_map_hash", { filename: name })

      if (remoteData[name].hash != hash) {
        map.status = WIC_Map_Status.OUTDATED
        console.log('hash mismatch', remoteData[name].hash, hash)
        throw new Error('hash mismatch')
      }

      map.hash = hash
      progress.off(progressId)
      job.info.push('done.')
      map.status = WIC_Map_Status.CURRENT
    })
  }
  busyLive = false

}

let busyCustom = false;
const queueCustom = []
const downloadCustomMap = async name => {
  if (queueCustom.includes(name)) return;
  queueCustom.push(name)
  _.find(state.value.mapsCustom, { name: name }).status = WIC_Map_Status.PENDING
  if (busyCustom) return;
  busyCustom = true

  while (queueCustom.length) {
    const name = queueCustom.shift()

    await manager.runJob(`Download ${name}`, async (job) => {
      const progressId = progress.on({ type: 'download-map-custom' }, (progress) => {
        job.progress = progress.percentage
      })

      const map = _.find(state.value.mapsCustom, { name: name })
      map.status = WIC_Map_Status.LOADING

      await invoke("download_map_custom", { map: name })
      map.status = WIC_Map_Status.CURRENT
      progress.off(progressId)
    })
  }
  busyCustom = false
}
// watch for action needed
const actionNeeded = computed(() => {
  return _.some(state.value.mapsLive, (map) => map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED)
    || _.some(state.value.mapsCustom, (map) => map.status == WIC_Map_Status.MISSING);
})

// computed sorted maps
const _mapsLive = computed(() => {
  return _.orderBy(state.value.mapsLive, [
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
const _mapsCustom = computed(() => {
  let filtered = _.filter(state.value.mapsCustom, map => map.status != WIC_Map_Status.CURRENT)
  return _.orderBy(filtered, [
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
  manager.runJob('Synchronize', async (job) => {
    const promisesCustom = _.map(state.value.mapsCustom, async (map) => {
      if (map.status == WIC_Map_Status.MISSING) {
        await downloadCustomMap(map.name)
      }
    })
    const promisesLive = _.map(state.value.mapsLive, async (map) => {
      if (map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED) {
        await downloadLiveMap(map.name)
      }
    })
    // combine arrays
    const promises = promisesCustom.concat(promisesLive)
    await Promise.all(promises)
  })
}

const _showUpload = ref(false)
onMounted(async () => {
  manager.runJob('Initialize', async (job) => {
    await init()
  })
})
</script>

<template>
  <div id="maps">
    <h2><span>MAPS</span> <button class="cta small secondary" v-if="!_showUpload"
        @click="_showUpload = true">Upload</button></h2>
    <maps-upload-vue v-if="_showUpload" />
    <div id="maps-live" class="maps-list-section">
      <div class="maps-list-container">
        <div class="maps-list-actions">
          <span class="cta" @click="synchronize" :class="{ inactive: !actionNeeded }">
            <iconDownload class="icon" />
            Download all missing/outdated
          </span>
        </div>
        <table class="maps-list" v-if="_mapsLive.length || _mapsCustom.length">
          <tr v-for="map in _mapsLive" :key="map.name">
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
              <span class="cta" @click="downloadLiveMap(map.name.toString())"
                v-if="map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED">
                <iconDownload class="icon" />
                Download
              </span>
              <div class="spinner-border" role="status" v-if="map.status == WIC_Map_Status.LOADING">
                <span class="sr-only">&nbsp;</span>
              </div>
              <iconCheck class="icon map-current" v-if="map.status == WIC_Map_Status.CURRENT" />
            </td>
          </tr>
          <tr v-for="(map, idx) in _mapsCustom" :key="'custom' + idx">
            <td colspan="5">{{ map.name }}</td>
            <td>{{ map.status }}</td>
            <td class="status">
              <span class="cta" @click="downloadCustomMap(map.name)"
                v-if="map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED">
                <iconDownload class="icon" />
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
      <jobs-vue :jobs="_jobs" id="maps-jobs" />
    </div>
  </div>
</template>

<style lang="scss">
#maps h2 {
  display: flex;
  flex-direction: row;

  span {
    flex: 1;
  }
}

.maps-list-section {

  .cta {
    text-wrap: nowrap;
  }

  display: flex;

  .spacer {
    flex: 1;
  }

  .maps-list-container {
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

  .maps-list-actions {

    height: 50px;

    display: flex;
    justify-content: flex-end;

    border-top-left-radius: 5px;
    border-top-right-radius: 5px;

    .cta {
      height: 50px;
      padding: 0 15px;
      height: 50px;
      line-height: 50px;
      border-width: 0;
      border-bottom-width: 1px;
      border-radius: 0;
      font-size: 18px;
      flex: 1;
      border-top-right-radius: 5px;
      border-top-left-radius: 10px;
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

  .maps-list {
    width: 100%;
    background: linear-gradient(to right, rgba(255, 255, 255, .2), rgba(255, 255, 255, .05));
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 5px;

    .cta {
      padding: 7px;
      font-size: 12px;
      border-bottom-width: 1px;
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