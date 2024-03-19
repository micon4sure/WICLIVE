import { invoke } from "@tauri-apps/api"


export default async () => {

  let config = {
    API_URL: ''
  }

  const environment = await invoke("get_environment")
  if (environment === 'development') {
    // config.API_URL = 'https://techtile.media:3243'
    config.API_URL = 'http://localhost:3243'
  } else {
    config.API_URL = 'https://techtile.media:3243'
  }

  return config;
}