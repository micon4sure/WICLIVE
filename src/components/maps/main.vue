<script setup lang="ts">
import _ from 'lodash'

import jobsVue from '../jobs.vue'
import axios from 'axios'

import { invoke } from "@tauri-apps/api/tauri";
import { computed, onMounted, ref, reactive } from 'vue';
import { listen } from '@tauri-apps/api/event';

import get_config from '../../get_config'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';
import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';
import iconClock from '@fortawesome/fontawesome-free/svgs/regular/clock.svg';

import mapsUploadVue from './upload.vue'

import { WIC_Map_Frontend, WIC_Map_Status } from '../../lib/wic-map'

import createJob, { Job, Job_Data } from '../../lib/wic-job';

let remoteData = {} as any

const _jobs: Job[] = reactive([])
const _queuedMaps: Set<string> = new Set()
let _isProcessingQueue = false

const state = ref({
  mapsLive: [] as WIC_Map_Frontend[]
})

const init = async () => {
  const CONFIG: any = await get_config()
  const local: Array<String> = await invoke("get_map_files");

  // init LIVE maps
  remoteData = (await axios.get(CONFIG.API_URL + '/maps/data')).data
  let promises = _.map(remoteData, async (map) => {
    map.name = map.name.toLowerCase()
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

const queueMapDownload = (name: string) => {
  // Prevent adding the same map twice
  if (_queuedMaps.has(name)) return

  _queuedMaps.add(name)
  _.find(state.value.mapsLive, { name }).status = WIC_Map_Status.QUEUED

  const job = createJob(`Download ${name}`, async (addInfo) => {
    const map = _.find(state.value.mapsLive, { name })
    map.status = WIC_Map_Status.DOWNLOADING

    await invoke("download_map_live", { map: name })

    addInfo('Compute hash...')
    const hash: string = await invoke("get_map_hash", { filename: name })

    if (remoteData[name].hash != hash) {
      map.status = WIC_Map_Status.OUTDATED
      _queuedMaps.delete(name)
      console.log('hash mismatch', remoteData[name].hash, hash)
      throw new Error('hash mismatch')
    }

    map.hash = hash
    addInfo('done.')
    map.status = WIC_Map_Status.CURRENT
    _queuedMaps.delete(name)
  }, 'download-map-live')

  _jobs.push(job)
  processQueue()
}

const processQueue = async () => {
  if (_isProcessingQueue) return
  _isProcessingQueue = true

  while (true) {
    const nextJob = _.find(_jobs, j => j.data.status === 'queued')
    if (!nextJob) break

    try {
      await nextJob.run()
    } catch (e) {
      console.error('Download failed', e)
    }
  }

  _isProcessingQueue = false
}

// watch for action needed
const actionNeeded = computed(() => {
  return _.some(state.value.mapsLive, (map) => map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED);
})

// computed sorted maps
const _mapsLive = computed(() => {
  return _.orderBy(state.value.mapsLive, [
    (map) => {
      if (map.status == WIC_Map_Status.MISSING) return 0;
      if (map.status == WIC_Map_Status.OUTDATED) return 1;
      if (map.status == WIC_Map_Status.DOWNLOADING) return 2;
      if (map.status == WIC_Map_Status.QUEUED) return 3;
      return 4
    },
    map => map.name
  ])
})

const synchronize = () => {
  if (!actionNeeded.value) return;
  _.each(state.value.mapsLive, (map) => {
    if (map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED) {
      queueMapDownload(map.name)
    }
  })
}

const _showUpload = ref(false)
onMounted(async () => {
  const job = createJob('Initialize', async (addInfo) => {
    await init()
  })
  _jobs.push(job)
  await job.run()
})
</script>

<template>
  <div id="maps" class="mb-5">
    <h2><span>MAPS</span> <button class="btn cta small secondary" v-if="!_showUpload"
        @click="_showUpload = true">Upload</button></h2>
    <maps-upload-vue v-if="_showUpload" />
    <div id="maps-live" class="maps-list-section">
      <div class="maps-list-container">
        <div class="maps-list-actions">
          <span class="btn cta" @click="synchronize" :class="{ inactive: !actionNeeded }">
            <iconDownload class="icon" />
            Download all missing/outdated
          </span>
        </div>
        <table class="maps-list" v-if="_mapsLive.length">
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
              <span class="btn cta" @click="queueMapDownload(map.name.toString())"
                v-if="map.status == WIC_Map_Status.MISSING || map.status == WIC_Map_Status.OUTDATED">
                <iconDownload class="icon" />
                Download
              </span>
              <iconClock class="icon map-queued" v-if="map.status == WIC_Map_Status.QUEUED" />
              <div class="spinner-border" role="status" v-if="map.status == WIC_Map_Status.DOWNLOADING">
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
    border-radius: 5px;
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
      flex: 1;
      border-top-right-radius: 5px;
      text-align: right;
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

      .cta {
        text-align: center;
      }
    }

    .icon.map-current {
      fill: #15a315;
      height: 3em;
    }

    .icon.map-queued {
      fill: #888;
      height: 3em;
    }
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
