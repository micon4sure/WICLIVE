<script setup lang="ts">
import _ from 'lodash'
import { defineProps } from 'vue'

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';
import iconXMark from '@fortawesome/fontawesome-free/svgs/solid/xmark.svg';

const props = defineProps({
  action: Object
})

const action = props.action as any
</script>

<template>
  <div :class="['action', action.status]">
    <span class="action-status">
      <iconCheck v-if="action.status === 'success'" />
      <iconXMark v-if="action.status === 'error'" />
      <div class="spinner-border" role="status" v-if="action.status == 'pending'">
        <span class="sr-only">&nbsp;</span>
      </div>
    </span>
    <div class="action-main">
      <div class="action-title">{{ action.title }}</div>
      <ul class="action-info" v-if="action.info.length">
        <li v-for="( info, idx ) in  action.info " :key="idx + info">{{ info }}</li>
      </ul>
      <div class="progress" v-if="action.progress && action.progress < 100">
        <div class="progress-bar bg-info" role="progressbar" :style="{ width: Math.floor(action.progress) + '%' }"
          :aria-valuenow="action.progress" aria-valuemin="0" aria-valuemax="100"></div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
div.action {
  display: flex;

  * {
    font-family: monospace;
  }

  .action.card-body {
    padding: 0 5px;
  }

  svg {
    width: 20px;
    height: 20px;
    fill: #fff;
    margin: 0 10px;
  }

  .spinner-border {
    width: 20px;
    height: 20px;
    color: red;
    margin: 0 10px;
    color: rgb(0, 162, 255)
  }

  &.error {
    svg {
      fill: #ff0000cc;
    }

    .action-title {
      background: #ff0000;
      padding: 5px;

    }
  }

  &.success svg {
    fill: #15a315;
  }

  .action-main {
    flex: 1;
  }

  .action-title,
  .action-info {
    font-family: monospace;
    font-size: 14px;
  }

  .action-info {
    background: rgba(0, 0, 0, .3);
    border-bottom-left-radius: 5px;
    border-bottom-right-radius: 5px;

    li {
      padding: 3px 10px;
      border-radius: 4px;
    }

  }
}

.progress {
  border-radius: 3px;
}
</style>