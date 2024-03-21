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
  <div class="action">
    <span class="action-status">
      <iconCheck v-if="action.status === 'success'" class="success" />
      <iconXMark v-if="action.status === 'error'" class="error" />
      <div class="spinner-border text-primary" role="status" v-if="action.status == 'pending'">
        <span class="sr-only">&nbsp;</span>
      </div>
    </span>
    <div class="action-main">
      <div class="action-title">{{ action.title }}</div>
      <ul class="action-info" v-if="action.info.length">
        <li v-for="( info, idx ) in  action.info " :key="idx + info">{{ info }}</li>
      </ul>
    </div>
  </div>
</template>

<style lang="scss">
div.action * {
  font-family: monospace;
}

div.action {
  display: flex;
}

.action .action.card-body {
  padding: 0 5px;
}

.action-status svg {
  width: 20px;
  height: 20px;
  fill: #fff;
  margin: 0 10px;

  &.success {
    fill: #148614;
  }
}

.action-header {
  font-size: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-direction: row;
}

.action-info {
  font-family: monospace;

  li {
    background: #000;
    padding: 3px 10px;
    border-radius: 4px;
  }
}
</style>