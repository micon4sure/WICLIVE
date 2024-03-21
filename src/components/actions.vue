<script setup lang="ts">
import _ from 'lodash'
import actionVue from './action.vue'
// actions as prop
import { defineProps, ref, onMounted, watch } from 'vue'
const props = defineProps({
  actions: Array
})
let actions = props.actions as Array<any>

const actionsSuccess = ref([]);
const actionsError = ref([]);
const actionsPending = ref([]);

watch(actions, (newVal) => {
  actionsSuccess.value = actions.filter((action) => action.status === 'success');
  actionsError.value = actions.filter((action) => action.status === 'error');
  actionsPending.value = actions.filter((action) => action.status === 'pending');
})
</script>

<template>
  <ul>
    <li v-for="(action, idx) in  actions " :key="idx + action.title" :class="['action', action.status]">
      <action-vue :action="action" />
    </li>
  </ul>
</template>

<style lang="scss"></style>