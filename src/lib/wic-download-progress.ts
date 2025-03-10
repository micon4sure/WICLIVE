import { listen } from '@tauri-apps/api/event';
import _ from 'lodash'

export default class WIC_DownloadProgress {

  private callbacks: { [key: number]: { callback: Function, filter: any } } = {}

  constructor() {
    listen('download-progress', this.onDownloadProgress.bind(this))
  }

  on(filter: any, callback: Function) {
    const key = performance.now()
    this.callbacks[key] = { 'callback': callback, filter }
    return key
  }
  off(key: number) {
    delete this.callbacks[key]
  }

  onDownloadProgress(event) {
    const payload = JSON.parse(event.payload)
    _.each(this.callbacks, (value, key) => {
      if (_.isMatch(payload, value.filter)) {
        value.callback(payload)
      }
    })
  }
}