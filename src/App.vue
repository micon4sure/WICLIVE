<script setup lang="ts">
</script>

<template>
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

h1 {
  display: flex;
  align-items: flex-end;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin: 0 -20px;
  margin-bottom: 20px;
  padding: 20px 40px;
}


h1 small {
  margin-left: 10px;
  font-size: 12px;
  color: #fff;
}

.card-header {
  background: linear-gradient(to right, #055479 0%, transparentize(#ce2e06, 1) 50%);
  border: none;
  border-top-left-radius: 5px;
  border-bottom-left-radius: 5px;
  padding-left: 15px;
  font-size: 20px;
}


.card {
  background: none;
  margin-bottom: 20px;
}

.card-body {
  background: linear-gradient(180deg, rgba(255, 255, 255, .1), rgba(255, 255, 255, 0.05));
  border-bottom-left-radius: 5px;
  border-bottom-right-radius: 5px;
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
