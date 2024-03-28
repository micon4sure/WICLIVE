<script setup lang="ts">
import _ from 'lodash'
import { useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api';
import { onMounted, reactive, ref } from 'vue'

import jobsVue from '../components/jobs.vue'

const state = reactive({
  jobs: []
})

const runJob = async (title, executor) => {
  const job = reactive({
    title,
    status: 'pending',
    info: [],
    progress: null
  })
  state.jobs.push(job)
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

let _step = ref("init")

onMounted(async () => {
  const route = useRoute()
  if (route.params.step == 'game') {
    _step.value = 'not_installed'
  }
  if (route.params.step == 'patch') {
    _step.value = 'not_patched'
  }
})

</script>

<template>
  <div id="init">
    <div v-if="_step === 'init'">
      <h3>Checking</h3>
      <jobsVue :jobs="state.jobs" />
    </div>
    <div v-else-if="_step === 'not_installed'">
      <div>
        <div class="card">
          <div class="card-header">Install from GOG Galaxy</div>
          <div class="card-body">
            <p>The easiest and fastest way to get the game is to
              <a href="https://www.gog.com/de/game/world_in_conflict_complete_edition" target="_blank">
                Buy and install the World in Conflict complete edition from Gog Galaxy
              </a>
            </p>
          </div>
        </div>
        <div class="card">
          <div class="card-header">Install from WIC LIVE</div>
          <div class="card-body">
            <p>Alternatively, you can
              <router-link to="/install" class="alternative">Install it here</router-link>
            </p>
          </div>
        </div>
      </div>
    </div>
    <div v-else-if="_step === 'not_patched'">
      <div class="card">
        <div class="card-header">World in Conflict is installed but not patched</div>
        <div class="card-body">
          <router-link to="/install">Automatically download and install the latest
            patches</router-link>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
#init {
  .card {
    border: 1px solid #333;
    border-top-right-radius: 5px;
    border-top-left-radius: 10px;
    border-bottom-left-radius: 10px;
    border-bottom-right-radius: 5px;
    background: rgba(255, 255, 255, .1);
    margin-bottom: 20px;
  }

  .card-header {
    background: #333;
    color: #fff;
  }

  .card-body a {
    font-family: eurostext;
    font-weight: 400;
    font-size: 24px;
    display: block;
    padding: 20px;
    border: 1px solid #333;
    border-radius: 5px;
    color: white;
    text-decoration: none;
    text-transform: uppercase;
    background-image: url('../assets/pattern-dots-primary.svg');

    &.alternative {
      background-image: url('../assets/pattern-dots-secondary.svg');
    }
  }

}
</style>
