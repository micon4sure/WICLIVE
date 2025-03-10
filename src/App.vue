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
  if (localStorage.getItem('do-install')) {
    router.push('/install/goes')
    return;
  }

  if (localStorage.getItem('initialized')) {
    return;
  }

  const installPath = await invoke('get_install_path')
  if (!installPath) {
    router.push('/init/game')
    return;
  }

  let version;
  try {
    version = await invoke('extract_game_version') as any;
  } catch (error) {
    console.log('forwarding to broken')
    router.push('/init/broken')
    return
  }
  const isPatched = version.patch == 1 && version.build == 1;
  if (!isPatched) {
    router.push('/init/patch')
  }
})
</script>

<template>
  <h1><a :href="home"><img src="./assets/wiclive.png" alt="WIC LIVE" /> <small>{{ _version }}</small></a></h1>
  <div id="container">
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

@font-face {
  font-family: "DIN_1451";
  src: url("./assets/bahnschrift.ttf");
  font-weight: 100;
}

body {
  background: url("./assets/pattern-stripes.svg");
}

#app {
  background: url("./assets/map.svg") no-repeat center center fixed;
}

h1 {
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin-bottom: 15px;
  padding: 25px;

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

  &,
  * {
    font-family: EUROSTEXT;
    font-size: 32px;
    letter-spacing: 1px;
    text-transform: uppercase;
    font-stretch: 80%;
  }
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

.card {
  background: transparentize($color: #fff, $amount: .8);
}

.card-header {
  font-family: EUROSTEXT;
  font-size: 26px;
  padding-left: 20px;
  letter-spacing: 1px;
  text-transform: uppercase;
  font-stretch: 80%;
  background: transparentize($color: #000, $amount: .5);

  &,
  & * {
    font-family: EUROSTEXT;
  }
}

.cta {
  font-family: "DIN_1451";
  letter-spacing: 3px;
  font-stretch: 110%;
  font-weight: 400;
  font-size: 18px;
  display: block;
  padding: 20px;
  border: 1px solid #333;
  border-radius: 5px;
  color: white;
  text-decoration: none;
  text-transform: uppercase;
  background-image: url('./assets/pattern-dots-primary.svg');
  border-bottom: 3px solid rgb(255, 136, 0);
  cursor: pointer;

  &.secondary {
    background-image: url('./assets/pattern-dots-secondary.svg');
    border-bottom: 3px solid rgb(0, 183, 255);
    font-size: 17px;
  }

  &.neutral {
    background-image: url('./assets/pattern-dots-neutral.svg');
    border-bottom: 3px solid rgb(0, 255, 13);
    font-size: 17px;
  }
}

.cta.small {
  font-size: 16px;
  padding: 10px;
  border-bottom-width: 1px;
}

.cta.inactive {
  border-color: black;
  background: #222;
  color: #666;
  background: url('./assets/pattern-dots-disabled.svg');

  svg {
    fill: #666;
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
