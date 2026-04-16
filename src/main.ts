import { createApp } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import App from './App.vue'
import './assets/styles/main.css'

const update = await check()
if (update) {
  await update.downloadAndInstall()
  await relaunch()
}

createApp(App).mount('#app')
