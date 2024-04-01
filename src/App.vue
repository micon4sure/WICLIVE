<script setup lang="ts">
import axios from 'axios';
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

import iconDownload from '@fortawesome/fontawesome-free/svgs/solid/download.svg';
import iconGithub from '@fortawesome/fontawesome-free/svgs/brands/github.svg';
import iconDiscord from '@fortawesome/fontawesome-free/svgs/brands/discord.svg';

const router = useRouter()
const home = router.resolve('/').href
</script>

<template>
  <h1>
    <a :href="home" id="link_home"><img src="./assets/wiclive.png" alt="WIC LIVE" /></a>
    <a href="https://github.com/micon4sure/WICLIVE/releases/latest/download/wiclive_x64-setup.exe"
      class="cta secondary">
      <iconDownload class="icon" />
      Download the latest release off github
    </a>
    <a href="https://github.com/micon4sure/WICLIVE"><icon-github class="icon external" /></a>
    <a href="https://discord.gg/xhbzTGcW"><icon-discord class="icon external" /></a>
  </h1>
  <div id="container">
    <main>
      <router-view />
    </main>
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


* {
  font-family: "eurostib";
}

body {
  background: linear-gradient(180deg, #082224 0%, #0d2f3f 100%);
  min-height: 100vh;
  background: url("./assets/pattern-stripes.svg");
}

#app {
  display: flex;
  flex-direction: column;
  background: url("./assets/map.svg") no-repeat center center fixed;
}

#container {
  padding: 20px;
  min-height: calc(100vh - 75px);
  flex: 1;
}

main {
  max-width: 1000px;
  margin: 0 auto;
}

a {
  color: rgb(57, 192, 255);
  text-decoration: none;

  &:hover {
    text-decoration: underline;
  }
}

h1 {
  display: flex;
  align-items: flex-end;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin: 0 -20px;
  margin-bottom: 20px;
  padding: 20px 40px;

  small {
    margin-left: 10px;
    font-size: 12px;
    color: #fff;
  }

  a#link_home {
    flex: 1;
  }

  a.cta {
    font-size: 16px;
  }

  >a>svg.external {
    padding: 0 10px;
    padding-bottom: 10px;
    fill: rgb(0, 174, 255);
  }
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

.card {
  background: transparentize($color: #fff, $amount: .8);
  margin-bottom: 10px;
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


.icon {
  fill: white;
  height: 1em;
  padding-bottom: 3px;
  height: 1em;
}

#showUpload {
  cursor: pointer;
  color: #fff;
  background-color: #000;
  padding: 10px;
  border-radius: 5px;
  margin: 10px;
  position: absolute;
  top: 10px;
  right: 60px;
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
