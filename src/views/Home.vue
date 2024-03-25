<script setup lang="ts">
</script>

<template>
  <div>
    <h1><img src="../assets/wiclive.png" alt="WIC LIVE" /></h1>
    <div class="card">
      <div class="card-header">What is WIC LIVE?</div>
      <div class="card-body">
        <p>In recent times, a lot of custom made maps have come out. Active players had to manually download them and
          keep them up to date.</p>
        <p> This has resulted in a lot of effort in our dedicated map makers being wasted because a majority of players
          didn't have the maps or an old version.</p>
        <p>WIC LIVE is the solution to this problem. It automatically downloads and updates community maps.</p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">Where can I get WIC LIVE?</div>
      <div class="card-body">
        <p>WIC LIVE can be downloaded directly off github. Just grab the setup.exe of <a
            :href="`https://github.com/micon4sure/WICLIVE/releases/latest`">the
            latest release</a>.
        </p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">How to use WIC LIVE?</div>
      <div class="card-body">
        <p>Using WIC LIVE is really simple. Just download it, install it and run it before you play. The use is
          self-explanatory.</p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">Do I have to run WIC LIVE before every game start?</div>
      <div class="card-body">
        <p>No. Map makers can only upload new maps and versions between tuesday noon and thursday noon. It is enough to
          update once for the weekend any time after thursday noon (UTC / Zulu time)
        </p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">Do I need WIC LIVE to play World in Conflict?</div>
      <div class="card-body">
        <p>Strictly NO! You can play World in Conflict without WIC LIVE. This tool is just for convinience. You can also
          just download the maps from <router-link to="/maps">the WIC LIVE map list</router-link>.
        </p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">Is this a virus?</div>
      <div class="card-body">
        <p>No, absolutely not. The entire project is open source, you can check it out on github, the builds are also
          made there.</p>
      </div>
    </div>
  </div>
</template>

<style lang="scss"></style>