<script setup lang="ts">
import _ from 'lodash'
import { useRoute, useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api';
import { onMounted, reactive, ref } from 'vue'

import jobsVue from '../jobs.vue'

const state = reactive({
  jobs: []
})

let _step = ref("init")

const route = useRoute()
const router = useRouter()
onMounted(async () => {
  if (route.params.step == 'game') {
    _step.value = 'not_installed'
  }
  if (route.params.step == 'patch') {
    _step.value = 'not_patched'
  }
})

const skip = () => {
  localStorage.setItem('initialized', 'true')
  router.push('/')
}
</script>

<template>
  <div id="init">
    <div v-if="_step === 'not_installed'">
      <div class="card">
        <div class="card-header">World in Conflict is not installed</div>
        <div class="card-body">
          <p>It appears that World in Conflict is not installed</p>
          <router-link to="/install" class="cta primary">Install World in Conflict</router-link>
          <button class="cta secondary" @click="skip">Skip installation</button>
        </div>
      </div>
    </div>
    <div v-else-if="_step === 'not_patched'">
      <div class="card">
        <div class="card-header">World in Conflict is installed but not patched</div>
        <div class="card-body">
          <router-link to="/install/goes" class="cta primary">Automatically download and install the latest
            patches</router-link>
          <button class="cta secondary" @click="skip">Skip</button>
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

    a:first-of-type {
      margin-bottom: 15px;
    }
  }

  .card-header {
    background: #333;
    color: #fff;
  }
}
</style>
