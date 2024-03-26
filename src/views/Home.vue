<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter()
const mapsRoute = router.resolve('/maps').href


const cards = [
  {
    name: 'what',
    header: 'What is WIC LIVE?',
    body: [`In recent times, a lot of custom made maps have come out. Active players had to manually download them and keep them up to date.`,
      `This has resulted in a lot of effort in our dedicated map makers being wasted because a majority of players didn't have the maps or an old version.`,
      `WIC LIVE is the solution to this problem. It automatically downloads and updates community maps.`]
  },
  {
    name: 'where',
    header: 'Where can I get WIC LIVE?',
    body: `WIC LIVE can be downloaded directly off github. Just grab <a href="https://github.com/micon4sure/WICLIVE/releases/latest/download/wiclive_x64-setup.exe">the latest release</a>.`
  },
  {
    name: 'every-time',
    header: 'Do I have to run WIC LIVE before every game start?',
    body: `No. Map makers can only upload new maps and versions between tuesday noon and thursday noon. It is enough to update once for the weekend any time after thursday noon (UTC / Zulu time)`
  },
  {
    name: 'need',
    header: 'Do I need WIC LIVE to play World in Conflict?',
    body: `Strictly NO! You can play World in Conflict without WIC LIVE. This tool is just for convinience. You can also just download the maps from <a href="${mapsRoute}">the WIC LIVE map list</a>.`
  },
  {
    name: 'virus',
    header: 'Is this a virus?',
    body: `No, absolutely not. The entire project is open source, you can check it out on github, the builds are also made there.`
  }
]

const _active = ref<string | null>(location.hash.substring(1) || 'what');
const setActive = (name: string) => {
  location.hash = name
  _active.value = name;
};
</script>

<template>
  <div id="home">
    <div v-for="card in cards" :key="card.name" :class="{ 'card': true, active: _active == card.name }"
      :name="card.name" @click="setActive(card.name)">
      <div class="card-header">{{ card.header }}</div>
      <div class="card-body">
        <p v-if="typeof card.body == 'string'" v-html="card.body" />
        <p v-else v-for="(p, idx) in card.body" :key="idx" v-html="p" />
      </div>
    </div>
  </div>
</template>

<style lang="scss">
#home {

  a {
    color: rgb(255, 136, 0);
    border-bottom: 1px solid rgb(255, 136, 0);

    &:hover {
      border-bottom: 2px solid rgb(255, 136, 0);
    }
  }

  .card-body {
    max-height: 0;
    overflow: hidden;
    padding: 0;
    transition: max-height 0.2s, padding .2s ease;

  }

  .card-header {
    border-bottom-left-radius: 5px;
    cursor: pointer;
  }

  .active {

    .card-body {
      height: auto;
      max-height: 200px;
      padding: 20px;

    }

    .card-header {
      border-bottom-left-radius: 0px;
    }
  }
}
</style>