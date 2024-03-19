<script setup lang="ts">
import _ from 'lodash'
// actions as prop
import { defineProps } from 'vue'
const props = defineProps({
  actions: Array
})
let actions = props.actions as Array<any>
</script>

<template>
  <ul id="actions">
    <li v-for="(action, idx) in  actions " :key="idx + action.title" :class="['action', 'card', action.status]">
      <div class="card-body">
        <div class="action-header">
          <span class="title card-title">{{ action.title }}</span>
          <div>{{ action.status }}
            <div class="spinner-border text-primary" role="status" v-if="action.status == 'pending'">
              <span class="sr-only">&nbsp;</span>
            </div>
          </div>
        </div>
        <ul class="action-info">
          <li v-for="( info, idx ) in  action.info " :key="idx + info">{{ info }}</li>
        </ul>
      </div>
    </li>
  </ul>
</template>

<style lang="scss">
.action {
  margin: 15px;

  &.success {
    border: 1px solid #248138;
    background: #174d23;
  }

  &.error {
    border: 1px solid #7e2424;
    background: #4d1717;
  }

  &.pending {
    border: 1px solid #000294;
    background: #242a36;
  }
}

.action-header {
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