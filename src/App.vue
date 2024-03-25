<script setup lang="ts">
</script>

<template>
  <div class="container">
    <router-view />
  </div>
  <footer>
    © 2024 WIC LIVE is NOT affiliated with Ubisoft or Ubisoft Massive.<br />
    © 2009 Ubisoft Entertainment. All Rights Reserved. World in Conflict, Ubisoft Massive the Ubisoft Massive logo,
    Ubisoft, Ubi.com and the Ubisoft logo are all trademarks of Ubisoft Entertainment in the US and/or other countries.
  </footer>
</template>

<style lang="scss">
.card {
  background: none;
  margin-bottom: 20px;
}

.card-header {
  background: linear-gradient(180deg, transparentize(#4b0a00, .5) 0%, transparentize(#82120b, .5) 100%);
  border: 1px solid #540a01;
  font-size: 20px;
}

.card-body {
  background: linear-gradient(180deg, rgba(255, 255, 255, .1), rgba(255, 255, 255, 0));
  border-bottom-left-radius: 5px;
  border-bottom-right-radius: 5px;
}

@font-face {
  font-family: "eurostib";
  src: url("./assets/eurostib.ttf");
}


h1 {
  display: flex;
  align-items: flex-end;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.1) 0%, rgba(0, 0, 0, 0.5) 100%);
  margin: 0 -40px;
  margin-bottom: 20px;
  padding: 20px 40px;
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

.card-header {
  background: linear-gradient(to right, #055479 0%, transparentize(#ce2e06, 1) 50%);
  border: none;
  border-top-left-radius: 5px;
  border-bottom-left-radius: 5px;
  padding-left: 15px;
}

#app {
  display: flex;
  flex-direction: column;
  background: url("./assets/pattern.svg");
}

* {
  font-family: "eurostib";
}

body {
  background: linear-gradient(180deg, #082224 0%, #0d2f3f 100%);
  min-height: 100vh;
}

.container {
  padding-top: 20px;
  min-height: calc(100vh - 75px);
  width: calc(100vw - 40px);
  max-width: calc(100vw - 40px);
  flex: 1;
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
