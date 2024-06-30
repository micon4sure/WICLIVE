<script setup lang="ts">
import { reactive, ref } from 'vue';
import mapsMainVue from '../components/maps/main.vue';
import configMainVue from '../components/config/main.vue';
import environmentMainVue from '../components/environment/main.vue';
import patchMainVue from '../components/patches/main.vue';

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';
import iconXMark from '@fortawesome/fontawesome-free/svgs/solid/xmark.svg';
import iconCircle from '@fortawesome/fontawesome-free/svgs/solid/circle.svg';
import iconCircleEmpty from '@fortawesome/fontawesome-free/svgs/regular/circle.svg';

import Bus from '../lib/Bus';

const _sectionState = reactive({
  maps: false,
  patches: false,
  config: false,
  environment: false,
})

const toggleSection = (section: string) => {
  _sectionState[section] = !_sectionState[section];
};

const _mapStatus = ref(null);

const onMapStatus = (status: string) => {
  _mapStatus.value = status;
};

const _mapActionNeeded = ref(null);
const onMapActionNeeded = (action: string) => {
  _mapActionNeeded.value = action;
  // if (action) {
  //   _sectionState.maps = true;
  // }
};

const _environment = ref('live');
const onSetEnvironment = (env: string) => {
  _environment.value = env;
};

const _patchStatus = ref(null);
const onPatchStatus = (status: string) => {
  _patchStatus.value = status;
};
const _patchActionNeeded = ref(null);
const onPatchActionNeeded = (action: string) => {
  _patchActionNeeded.value = action;
  // if (action) {
  //   _sectionState.patch = true;
  // }
};

const bus = new Bus();
const _showUploadMap = ref(true);
const uploadMapClicked = (event) => {
  event.preventDefault();
  event.stopPropagation();
  _sectionState.maps = true;
  bus.emit('upload-map');
  _showUploadMap.value = false;
};
const _showUploadPatch = ref(true);
const uploadPatchClicked = (event) => {
  event.preventDefault();
  event.stopPropagation();
  _sectionState.patches = true;
  bus.emit('upload-patch');
  _showUploadPatch.value = false;
};

const _patchesEnabled = ref(false);
const onPatchChange = (status) => {
  _patchesEnabled.value = status;
};
</script>

<template>
  <div id="accordion">
    <h2 @click="toggleSection('maps')">
      <span>MAPS</span>

      <div class="section-status">
        <small class="text-muted">
          / need action..:
          <div class="section-status-spinner" v-if="_mapStatus == 'init'">
            <div class="spinner-border" role="status">
              <span class="sr-only">&nbsp;</span>
            </div>
          </div>

          <span v-if="_mapStatus == 'initDone'">
            <span v-if="_mapActionNeeded">
              YES
              <iconCircle class="warning" />
            </span>
            <span v-else>
              no
              <iconCircleEmpty class="success" />
            </span>
          </span>
        </small>
      </div>
      <small v-if="_showUploadMap">
        <button class="btn cta small secondary" @click="uploadMapClicked">upload</button>
      </small>
    </h2>
    <div id="collapseMaps" class="collapse" :class="{ 'show': _sectionState.maps }" aria-labelledby="headingMaps">
      <div class="card-body">
        <maps-main-vue @onStatus="onMapStatus" @onActionNeeded="onMapActionNeeded" :bus="bus" />
      </div>
    </div>

    <h2 @click="toggleSection('patches')">
      <span>Patches</span>

      <div class="section-status">
        <small class="text-muted">
          / need action..:
          <div class="section-status-spinner" v-if="_patchStatus == 'init'">
            <div class="spinner-border" role="status">
              <span class="sr-only">&nbsp;</span>
            </div>
          </div>

          <span v-if="_patchStatus == 'initDone'">
            <span v-if="_patchActionNeeded">
              YES
              <iconCircle class="warning" />
            </span>
            <span v-else>
              no
              <iconCircleEmpty class="success" />
            </span>
          </span>
          <span class="m-2">/
            {{ _patchesEnabled ? 'enabled' : 'disabled' }}
          </span>
        </small>
      </div>
      <small v-if="_showUploadPatch">
        <button class="btn cta small secondary" @click="uploadPatchClicked">upload</button>
      </small>
    </h2>
    <div id="collapseConfig" class="collapse mb-3" :class="{ 'show': _sectionState.patches }"
      aria-labelledby="headingConfig">
      <div class="card-body">
        <patch-main-vue @onStatus="onPatchStatus" @onActionNeeded="onPatchActionNeeded" :bus="bus"
          @onChange="onPatchChange" />
      </div>
    </div>

    <h2 @click="toggleSection('environment')">Environment
      <div class="section-status">
        <small>: {{ _environment }}</small>
      </div>
    </h2>
    <div id="collapseConfig" class="collapse" :class="{ 'show': _sectionState.environment }"
      aria-labelledby="headingConfig">
      <div class="card-body">
        <environment-main-vue @on-set-environment="onSetEnvironment" />
      </div>
    </div>

    <h2 @click="toggleSection('config')">Config</h2>
    <div id="collapseConfig" class="collapse" :class="{ 'show': _sectionState.config }" aria-labelledby="headingConfig">
      <div class="card-body">
        <config-main-vue />
      </div>
    </div>
  </div>
</template>

<style lang="scss">
/* Add any custom styles if needed */
.collapse {
  display: none;
}

.collapse.show {
  display: block;
}

small,
small span {
  font-size: 12px;
  line-height: 32px;
}

small {

  div {
    display: inline-block;
  }

  .spinner-border {
    width: 32px;
    height: 32px;
    font-size: 10px;
  }

  svg {
    height: 12px;

    &.success {
      fill: green;
    }

    &.warning {
      fill: orange;
    }
  }
}
</style>
<style scoped lang="scss">
h2 {
  cursor: pointer;
  display: flex;

  .section-status {
    flex: 1;
    padding-left: 10px;
  }
}
</style>
