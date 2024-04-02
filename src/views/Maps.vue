<script setup lang="ts">
import _ from 'lodash'
import axios from 'axios'
import { onMounted, ref, type Ref } from 'vue'

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';


const _maps = ref([] as any[])
const _sort: Ref<string> = ref('date')
const _sortDirection: Ref<'asc' | 'desc'> = ref('desc')

onMounted(async () => {
  const response = await axios.get('https://techtile.media:3243/maps/data')

  _maps.value = response.data

  _.each(_maps.value, (map) => {
    map.size = (map.size / 1024 / 1024).toFixed(2) + ' MB'
  })
})


const setSort = (sort: string) => {
  if (_sort.value === sort) {
    _sortDirection.value = _sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    _sort.value = sort
    _sortDirection.value = 'asc'
  }

  _maps.value = _.orderBy(_maps.value, [_sort.value], [_sortDirection.value])
}
</script>

<template>
  <h2>MAPS</h2>
  <table>
    <tr>
      <th></th>
      <th @click="setSort('name')">Name <span v-if="_sort === 'name'">{{ _sortDirection === 'asc' ? '↑' :
        '↓' }}</span></th>
      <th @click="setSort('version')">Version <span v-if="_sort === 'version'">{{ _sortDirection === 'asc' ?
        '↑' : '↓' }}</span></th>
      <th @click="setSort('uploader')">Author <span v-if="_sort === 'uploader'">{{ _sortDirection === 'asc'
        ? '↑' : '↓' }}</span></th>
      <th @click="setSort('date')">Date <span v-if="_sort === 'date'">{{ _sortDirection === 'asc' ? '↑' :
        '↓' }}</span></th>
      <th @click="setSort('size')">Size <span v-if="_sort === 'size'">{{ _sortDirection === 'asc' ? '↑' :
        '↓' }}</span></th>
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

  th {
    cursor: pointer;
  }

  td {
    &:first-child {
      padding-left: 10px;
    }

    padding-top: 15px;
    padding-bottom: 15px;
  }

  tr {
    border-bottom: 1px solid rgba(255, 255, 255, .2);
    background: rgba(255, 255, 255, .1);

    &:nth-child(odd) {
      background: rgba(255, 255, 255, .2);
    }

    &:first-child {
      background: none;
    }
  }

  .cta {
    display: inline;
  }
}
</style>