<script setup lang="ts">
</script>

<template>
  <div class="container">
    <h1><img src="./assets/wiclive.png" alt="WIC LIVE" /></h1>
    <div class="card">
      <div class="card-header">What is WIC LIVE?</div>
      <div class="card-body">
        <p>In recent times, a lot of custom made maps have come out. Active players had to manually download them and
          keep them up to date.</p>
        <p> This has resulted in a lot of effort in our dedicated map makers being wasted because a majority of players
          didn't have the maps or an old version</p>
        <p>WIC LIVE is the solution to this problem. It automatically downloads and updates community maps.</p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">Where can I get WIC LIVE?</div>
      <div class="card-body">
        <p>WIC LIVE can be downloaded directly off github. Just grab the setup .exe of <a
            :href="`https://github.com/micon4sure/WICLIVE/releases/latest`">the
            latest release</a>.
        </p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">How to use WIC LIVE?</div>
      <div class="card-body">
        <p>Using WIC LIVE is really simple. Just download it, install it and run it before you play. There is a feature
          planned that automatically
          starts World in Conflict after maps are synchronized. Coming soon.</p>
      </div>
    </div>
    <div class="card">
      <div class="card-header">Do I need WIC LIVE to play World in Conflict?</div>
      <div class="card-body">
        <p>Strictly NO! You can play World in Conflict without WIC LIVE. You will only need this to play on a server
          with custom maps.</p>
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
