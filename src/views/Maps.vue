<script setup lang="ts">
import _ from 'lodash'
import axios from 'axios'
import { onMounted, ref } from 'vue'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';


const _maps = ref([] as any[])
onMounted(async () => {
  const response = await axios.get('https://techtile.media:3243/maps/data')
  console.log(response.data)

  _maps.value = _.orderBy(response.data, ['date'], ['desc'])

  _.each(_maps.value, (map) => {
    map.size = (map.size / 1024 / 1024).toFixed(2) + ' MB'
  })
})
</script>

<template>
  <h2>MAPS</h2>
  <table>
    <tr>
      <th></th>
      <th>Name</th>
      <th>Version</th>
      <th>Author</th>
      <th>Date</th>
      <th>Size</th>
    </tr>
    <tr v-for="map in _maps" :key="map.id">
      <td><a :href="'https://techtile.media:3243/maps/download/' + map.name" class="cta small">
          <iconDownload class="icon" />
          Download
        </a></td>
      <td>{{ map.name }}</td>
      <td>v{{ map.version }}</td>
      <td>{{ map.uploader }}</td>
      <td>{{ map.date }}</td>
      <td>{{ map.size }}</td>
    </tr>
  </table>
</template>

<style lang="scss">
table {
  width: 100%;
  border-collapse: collapse;
  max-width: 800px;

  td {
    padding-top: 10px;
    padding-bottom: 10px;
  }

  tr {
    border-bottom: 1px solid rgba(255, 255, 255, .2);

    &:first-of-type {
      border-bottom: none;
    }
  }

  .cta {
    display: inline;
  }
}
</style>