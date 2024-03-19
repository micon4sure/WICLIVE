import { invoke } from "@tauri-apps/api"


export default async () => {
  return await invoke("get_config")
}