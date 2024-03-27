<script setup lang="ts">
import { invoke } from '@tauri-apps/api';
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router';
const _version = ref('');
onMounted(async () => {
  const config: any = await invoke('get_config')
  _version.value = config.VERSION
})


const router = useRouter()
const home = router.resolve('/').href

console.log(router.currentRoute.value.path)

if (router.currentRoute.value.path === '/') {
  router.push('/init')
}
</script>

<template>
  <div id="container">
    <h1><a :href="home"><img src="./assets/wiclive.png" alt="WIC LIVE" /> <small>{{ _version }}</small></a></h1>
    <router-view />
  </div>
  <footer>
    This project is not affiliated, associated, authorized, endorsed by, or in any way officially connected with MASSIVE
    Entertainment or UBISOFT, or any of their subsidiaries or affiliates.<br />
    All trademarks and registered trademarks are the property of their respective owners. The use of these names,
    trademarks, and brands does not imply endorsement.
  </footer>
</template>

<style lang="scss">
@font-face {
  font-family: "eurostib";
  src: url("./assets/eurostib.ttf");
}

body {
  background: url("./assets/pattern-stripes.svg");
}

#app {
  background: url("./assets/map.svg") no-repeat center center fixed;
}

h1 {
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin: 0 -20px;
  margin-bottom: 20px;
  padding: 20px;

  a {
    display: flex;
    align-items: flex-end;
    text-decoration: none;
  }
}


h1 small {
  margin-left: 10px;
  font-size: 12px;
  color: #fff;
}

h2 {
  background: linear-gradient(to right, #055479 0%, transparentize(#ce2e06, 1) 50%);
  border-top-left-radius: 5px;
  border-bottom-left-radius: 5px;
  padding-left: 15px;
}

* {
  font-family: "eurostib";
}

#container {
  padding: 20px;
  margin: 0px;
  width: calc(100vw - 20px);
  min-height: calc(100vh - 75px);
  flex: 1;
}


#showUpload {
  cursor: pointer;
  color: #fff;
  background-color: rgba(0, 0, 0, 0.2);
  padding: 10px;
  border-radius: 5px;
  position: absolute;
  top: 40px;
  right: 20px;
}

footer {
  color: #fff;
  text-align: center;
  padding: 10px;
  bottom: 0;
  width: calc(100vw - 100px);
  height: 75px;
  margin: 0 50px;
  font-size: 11px;
}
</style>
