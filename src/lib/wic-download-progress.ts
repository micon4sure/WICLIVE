import { listen } from '@tauri-apps/api/event';
import _ from 'lodash'

export default class WIC_DownloadProgress {

  private callbacks: { [key: number]: { callback: Function, map: string } } = {}

  constructor() {
    listen('download-progress', this.onDownloadProgress.bind(this))
  }

  on(map: any, callback: Function) {
    const key = performance.now()
    this.callbacks[key] = { 'callback': callback, map: map }
    return key
  }
  off(key: number) {
    delete this.callbacks[key]
  }

  onDownloadProgress(event) {
    const payload = JSON.parse(event.payload)
    _.each(this.callbacks, (callback, key) => {
      if (callback.map != payload.map)
        return
      callback.callback(payload)
    })
  }
}