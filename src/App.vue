<script setup lang="ts">
import axios from 'axios';
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter()
const home = router.resolve('/').href
</script>

<template>
  <h1>
    <a :href="home" id="link_home"><img src="./assets/wiclive.png" alt="WIC LIVE" /></a>
    <a href="https://github.com/micon4sure/WICLIVE/releases/latest/download/wiclive_x64-setup.exe" class="cta">Download
      the latest release off github</a>
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
  background: none;
  margin-bottom: 20px;
}

.card-header {
  background: linear-gradient(to right, #055479 0%, transparentize(#ce2e06, 1) 50%);
  font-family: EUROSTEXT;
  font-size: 26px;
  padding-left: 20px;
  letter-spacing: 1px;
  text-transform: uppercase;
  font-stretch: 80%;
}

.card-body {
  background: linear-gradient(180deg, rgba(255, 255, 255, .1), rgba(255, 255, 255, 0.05));
  border-bottom-left-radius: 5px;
  border-bottom-right-radius: 5px;
}


a.cta,
button.cta {
  font-family: eurostext;
  font-weight: 400;
  font-size: 24px;
  display: block;
  padding: 20px;
  border: 1px solid #333;
  border-radius: 5px;
  color: white;
  text-decoration: none;
  text-transform: uppercase;
  background-image: url('/pattern-dots-primary.svg');
  border-bottom: 1px solid rgb(255, 136, 0);

  &.secondary {
    background-image: url('/pattern-dots-secondary.svg');
  }


  &:hover {
    text-decoration: underline;
  }

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
