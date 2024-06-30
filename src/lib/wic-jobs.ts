import { reactive } from 'vue';

import { listen } from '@tauri-apps/api/event';
import _ from 'lodash'

class WIC_DownloadProgress {

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
class WIC_JobManager {

  private _jobs = reactive([])

  getJobs() {
    return this._jobs;
  }

  clearJobs() {
    this._jobs.splice(0, this._jobs.length);
  }

  async runJob(title, executor) {
    const job = reactive({
      title,
      status: 'pending',
      info: [],
      progress: null
    });
    this._jobs.push(job);
    let result = null;
    try {
      console.log('running job', title);
      result = await executor(job);
      job.status = 'success';
    } catch (error) {
      job.status = 'error';
      job.info.push(error);
    }
    return result;
  }
}

const mapManager = new WIC_JobManager();
const patchManager = new WIC_JobManager();
const progress = new WIC_DownloadProgress();


export default {
  mapManager,
  patchManager,
  progress
}
