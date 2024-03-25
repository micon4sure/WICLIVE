<script setup lang="ts">
import _ from 'lodash'
import axios from 'axios'
import { onMounted, ref } from 'vue'

const _maps = ref([] as any[])
onMounted(async () => {
  const response = await axios.get('https://techtile.media:3243/maps/data')
  console.log(response.data)

  _maps.value = _.orderBy(response.data, ['name'], ['asc'])

  _.each(_maps.value, (map) => {
    map.size = (map.size / 1024 / 1024).toFixed(2) + ' MB'
  })
})
</script>

<template>
  <div>
    <h1><img src="../assets/wiclive.png" alt="WIC LIVE" /></h1>
    <h2>MAPS</h2>
    <table>
      <tr>
        <th>Download</th>
        <th>Map Name</th>
        <th>Version</th>
        <th>Author</th>
        <th>Date</th>
        <th>Size</th>
      </tr>
      <tr v-for="map in _maps" :key="map.id">
        <td><a :href="'https://techtile.media:3243/maps/download/' + map.name">Download</a></td>
        <td>{{ map.name }}</td>
        <td>v{{ map.version }}</td>
        <td>{{ map.uploader }}</td>
        <td>{{ map.date }}</td>
        <td>{{ map.size }}</td>
      </tr>
    </table>
  </div>
</template>

<style lang="scss">
table {
  margin: 0 auto;
  width: 100%;
  border-collapse: collapse;
  max-width: 800px;
}
</style>