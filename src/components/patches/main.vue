<script setup lang="ts">
import _ from 'lodash'

import jobsVue from '../jobs.vue'
import axios from 'axios'

import { invoke } from "@tauri-apps/api/tauri";
import { computed, onMounted, reactive, ref, watch, emit } from 'vue';

import get_config from '../../get_config'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';
import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';

import patchUploadVue from './upload.vue'

import { WIC_Patch_Frontend, WIC_Content_Status } from '../../lib/wic-content'

import wicJobs from '../../lib/wic-jobs';
import Bus from '../../lib/Bus';

const props = defineProps({
  bus: Bus
})

const emit = defineEmits(['onStatus', 'onActionNeeded', 'onChange'])

const _patchesEnabled = ref(false)
if (localStorage.getItem('patches-enabled') == 'true') {
  _patchesEnabled.value = true
} else {
  _patchesEnabled.value = false
}
emit('onChange', _patchesEnabled.value)


const toggleEnablePatches = () => {
  _patchesEnabled.value = !_patchesEnabled.value
  localStorage.setItem('patches-enabled', _patchesEnabled.value.toString())
  if (_patchesEnabled.value) {
    invoke('disable_patches')
  } else {
    invoke('enable_patches')
  }
  emit('onChange', _patchesEnabled.value)
}


const manager = wicJobs.patchManager
const progress = wicJobs.progress
const _jobs = manager.getJobs()
manager.clearJobs();

let remoteData = {} as any

const state = ref({
  patches: [] as WIC_Patch_Frontend[]
})

const init = async () => {
  emit('onStatus', 'onChange')

  const CONFIG: any = await get_config()
  const local: Array<String> = await invoke("get_patch_files", { patchesEnabled: _patchesEnabled.value });

  // init patches
  const remote = (await axios.get(CONFIG.API_URL + '/patches/data')).data
  remoteData = remote

  console.log('remote', remote)

  let promises = _.map(remote, async (patch) => {
    let status: WIC_Content_Status;
    if (!_.includes(local, patch.name)) {
      status = WIC_Content_Status.MISSING
    } else {
      const hash = await invoke("get_patch_hash", { filename: patch.name, patchesEnabled: _patchesEnabled.value })
      if (hash != patch.hash) {
        status = WIC_Content_Status.OUTDATED
      } else {
        status = WIC_Content_Status.CURRENT
      }
    }

    let size = patch.size / 1024 / 1024
    size = Math.round(size * 100) / 100
    const data = {
      name: patch.name,
      status: status,
      date: patch.date,
      uploader: patch.uploader,
      version: patch.version,
      size
    } as WIC_Patch_Frontend

    state.value.patches.push(data)
  })
  await Promise.all(promises)

  emit('onStatus', 'initDone')
}

const queue = []
let busy = false
const downloadPatch = async (name: string) => {
  if (queue.includes(name)) return;
  queue.push(name)
  _.find(state.value.patches, { name: name }).status = WIC_Content_Status.PENDING
  if (busy) return;
  busy = true

  while (queue.length) {
    const name = queue.shift()

    await manager.runJob(`Download ${name}`, async (job) => {
      const progressId = progress.on({ type: 'download-patch' }, (progress) => {
        job.progress = progress.percentage
      })

      const patch = _.find(state.value.patches, { name: name })
      patch.status = WIC_Content_Status.LOADING

      await invoke("download_patch", { patch: name })

      job.info.push('Compute hash...')
      const hash: string = await invoke("get_patch_hash", { filename: name })

      if (remoteData[name].hash != hash) {
        patch.status = WIC_Content_Status.OUTDATED
        console.log('hash mismatch', remoteData[name].hash, hash)
        throw new Error('hash mismatch')
      }

      patch.hash = hash
      progress.off(progressId)
      job.info.push('done.')
      patch.status = WIC_Content_Status.CURRENT
    })
  }
  busy = false

}

// watch for action needed
const actionNeeded = computed(() => {
  const needed = _.some(state.value.patches, (patch) => patch.status == WIC_Content_Status.MISSING || patch.status == WIC_Content_Status.OUTDATED)

  emit('onActionNeeded', needed)
  return needed;
})

// computed sorted patches
const _patches = computed(() => {
  return _.orderBy(state.value.patches, [
    (patch) => {
      if (patch.status == WIC_Content_Status.MISSING) return 0;
      if (patch.status == WIC_Content_Status.OUTDATED) return 1;
      if (patch.status == WIC_Content_Status.LOADING) return 2;
      if (patch.status == WIC_Content_Status.PENDING) return 3;
      return 4
    },
    patch => patch.name
  ])
})

const synchronize = async () => {
  if (!actionNeeded.value) return;
  manager.runJob('Synchronize', async (job) => {
    const promises = _.map(state.value.patches, async (patch) => {
      if (patch.status == WIC_Content_Status.MISSING) {
        await downloadPatch(patch.name)
      }
    })
    await Promise.all(promises)
  })
}

const _showUpload = ref(false)
onMounted(async () => {
  manager.runJob('Initialize Patch Data', async (job) => {
    console.log('HERE')
    await init()
  })
})

props.bus.on('upload-patch', () => {
  console.log('UPLOAD PATCH')
  _showUpload.value = true
})
</script>

<template>
  <div id="patches">
    <patch-upload-vue v-if="_showUpload" />
    <button class="btn cta small m-3" @click="toggleEnablePatches" v-if="!_patchesEnabled">Enable
      Patches</button>
    <button class="btn cta small secondary m-3" @click="toggleEnablePatches" v-else>Disable Patches</button>
    <div id="patches-live" class="patches-list-section">
      <div class="patches-list-container">
        <div class="patches-list-actions">
          <span class="cta" @click="synchronize" :class="{ inactive: !actionNeeded }">
            <iconDownload class="icon" />
            Download all missing/outdated
          </span>
        </div>
        <table class="patches-list" v-if="_patches.length">
          <tr v-for="patch in _patches" :key="patch.name">
            <th>
              {{ patch.name }}
            </th>
            <td>
              v{{ patch.version }}
            </td>
            <td>
              {{ patch.uploader }}
            </td>
            <td>
              {{ patch.date }}
            </td>
            <td>
              {{ patch.size }} MB
            </td>
            <td>
              <span v-if="patch.status != WIC_Content_Status.CURRENT">{{ patch.status }}</span>
            </td>
            <td class="status">
              <span class="cta" @click="downloadPatch(patch.name.toString())"
                v-if="patch.status == WIC_Content_Status.MISSING || patch.status == WIC_Content_Status.OUTDATED">
                <iconDownload class="icon" />
                Download
              </span>
              <div class="spinner-border" role="status" v-if="patch.status == WIC_Content_Status.LOADING">
                <span class="sr-only">&nbsp;</span>
              </div>
              <iconCheck class="icon patch-current" v-if="patch.status == WIC_Content_Status.CURRENT" />
            </td>
          </tr>
        </table>
        <small v-else class="p-5">no patches</small>
      </div>
      <jobs-vue :jobs="_jobs" id="patches-jobs" />
    </div>
  </div>
</template>

<style lang="scss">
#show-upload {
  display: flex;
  justify-content: flex-end;
}

.patches-list-section {

  .cta {
    text-wrap: nowrap;
  }

  display: flex;

  .spacer {
    flex: 1;
  }

  .patches-list-container {
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

  .patches-list-actions {

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

  .patches-list {
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

    .icon.patch-current {
      fill: #15a315;
      height: 1.5em;

    }
  }

  #patches-jobs {
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