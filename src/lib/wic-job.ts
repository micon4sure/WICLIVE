import { reactive } from 'vue';

import WIC_DownloadProgress from './wic-download-progress';

export type Job_Info = {
  text: string;
  highlight?: boolean;
};

export type Job_Data = {
  title: string;
  status: 'queued' | 'running' | 'success' | 'error' | 'cancelled';
  info: Job_Info[];
  progress: number;
};

export type Job = {
  data: Job_Data;
  run: () => Promise<void>;
};

const progress = new WIC_DownloadProgress();
const createJob = (title: string, action: Function, progressFilter: any = null): Job => {
  const data = reactive<Job_Data>({
    title,
    status: 'queued',
    info: [],
    progress: 0,
  });

  const run = async () => {
    let progressListenerId: number | null = null;

    if (progressFilter) {
      const filter = typeof progressFilter === 'string' ? { type: progressFilter } : progressFilter;
      progressListenerId = progress.on(filter, (payload) => {
        const percentage = (payload as any).percentage ?? (payload as any).progress;
        if (typeof percentage === 'number') {
          data.progress = percentage;
        }
      });
    }

    try {
      data.status = 'running';
      await action((info: string) => data.info.push({ text: info, highlight: false }), data);
      // await new Promise(r => setTimeout(r, 100)); // allow UI to update
      data.status = 'success';
    } catch (e) {
      console.error('Job error', e);
      data.status = 'error';
      data.info.push({ text: String(e), highlight: false });
      throw e;
    } finally {
      if (progressListenerId !== null) {
        progress.off(progressListenerId);
      }
    }
  };

  return { data, run };
};

export default createJob