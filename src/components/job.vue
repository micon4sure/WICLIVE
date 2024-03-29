<script setup lang="ts">
import _ from 'lodash'
import { defineProps } from 'vue'

import iconCheck from '@fortawesome/fontawesome-free/svgs/solid/check.svg';
import iconXMark from '@fortawesome/fontawesome-free/svgs/solid/xmark.svg';

const props = defineProps({
  job: Object
})

const job = props.job as any
</script>

<template>
  <div :class="['job', job.status]">
    <span class="job-status">
      <iconCheck v-if="job.status === 'success'" />
      <iconXMark v-if="job.status === 'error'" />
      <div class="spinner-border" role="status" v-if="job.status == 'pending'">
        <span class="sr-only">&nbsp;</span>
      </div>
    </span>
    <div class="job-main">
      <div class="job-title">{{ job.title }}</div>
      <ul class="job-info" v-if="job.info.length">
        <li v-for="( info, idx ) in  job.info " :key="idx + info">{{ info }}</li>
      </ul>
      <div class="progress" v-if="job.progress && job.progress < 100">
        <div class="progress-bar bg-info" role="progressbar" :style="{ width: Math.floor(job.progress) + '%' }"
          :aria-valuenow="job.progress" aria-valuemin="0" aria-valuemax="100"></div>
      </div>
    </div>
  </div>
</template>

<style lang="scss">
div.job {

  ul,
  li {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  display: flex;

  * {
    font-family: 'Consolas', monospace;
  }

  .job.card-body {
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

    .job-title {
      background: #ff0000;
      padding: 5px;

    }
  }

  &.success svg {
    fill: #15a315;
  }

  .job-main {
    flex: 1;
  }

  .job-title,
  .job-info {
    font-size: 14px;
  }

  .job-info {
    background: rgba(0, 0, 0, .3);
    border-radius: 5px;
    padding: 5px;

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