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

onMounted(async () => {
  if (localStorage.getItem('force-url')) {
    console.log('force-url', localStorage.getItem('force-url'))
    router.push(localStorage.getItem('force-url') as string)
    localStorage.removeItem('force-url')
    return;
  }
  const installPath = await invoke('get_install_path')
  if (!installPath) {
    router.push('/init/game')
  }
  const version = await invoke('extract_game_version') as any;
  const isPatched = version.patch == 1 && version.build == 1;
  if (!isPatched) {
    router.push('/init/patch')
  }
})
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

@font-face {
  font-family: "eurostext";
  src: url("./assets/ESTEXTR.ttf");
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
  padding: 10px 15px;

  font-family: EUROSTEXT;
  font-size: 32px;
  letter-spacing: 1px;
  text-transform: uppercase;
  font-stretch: 80%;

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

.card-header {
  font-family: EUROSTEXT;
  font-size: 32px;
  padding-left: 20px;
  letter-spacing: 1px;
  text-transform: uppercase;
  font-stretch: 80%;
}

.btn-container {
  flex: 1;
  cursor: pointer;
  display: inline-block;
  justify-content: space-between;
  align-items: center;
  height: 35px;
  border: none;
  border-radius: 5px;
  // background: linear-gradient(0deg, #791c05 0%, #ce2e06 100%);
  height: 35px;
  line-height: 35px;
  padding: 0 10px;
  text-align: left;
  text-wrap: nowrap;

  &.primary {
    background-image: url('./assets/pattern-dots-primary.svg');
  }

  &.secondary {
    background: #333;

    color: #aaa;

    svg {
      fill: #aaa;
    }
  }

  button {
    height: 35px;
    line-height: 15px;
    border: none;
    background: transparent;
  }
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
