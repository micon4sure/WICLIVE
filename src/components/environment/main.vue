<script setup lang="ts">
import _ from 'lodash';

import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api';

const emit = defineEmits(['onSetEnvironment'])

const _environment = ref(null);

const setEnvironment = (env: string) => {
  _environment.value = env;

  invoke('environment_set', { environment: env });

  emit('onSetEnvironment', env);
  localStorage.setItem('environment', env);
};

onMounted(() => {
  const current = localStorage.getItem('environment');
  if (current) {
    _environment.value = current;
    emit('onSetEnvironment', current);
  }
})

</script>

<template>
  <div id="environment">
    <div class="card">
      <div class="card-header">Set Environment</div>
      <div class="card-body">
        <div class="btn-group" role="group" aria-label="Environment Buttons">
          <button type="button" class="btn primary btn-outline-primary"
            :class="{ 'active-env': _environment === 'development' }" @click="setEnvironment('development')">
            Development
          </button>
          <button type="button" class="btn warning btn-outline-warning"
            :class="{ 'active-env': _environment === 'testing' }" @click="setEnvironment('testing')">
            Testing
          </button>
          <button type="button" class="btn success btn-outline-success"
            :class="{ 'active-env': _environment === 'live' }" @click="setEnvironment('live')">
            Live
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.btn {
  background-color: transparent;
  color: white;
  display: inline-block;
  min-width: 100px;
  ;
}

.primary.active-env {
  color: white;
  background-color: #007bff;
}

.warning.active-env {
  color: black;
  background-color: #ffc107;
}

.success.active-env {
  color: white;
  background-color: #28a745;
}
</style>
