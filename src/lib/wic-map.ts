export enum WIC_Map_Status {
  MISSING = 'missing',
  OUTDATED = 'outdated',
  QUEUED = 'queued',
  DOWNLOADING = 'downloading',
  CURRENT = 'current',
}

export interface WIC_Map_Backend {
  name: string;
  hash: string;
  size: number;
  date: string;
  version: number;
  uploader: string;
}
export interface WIC_Map_Frontend extends WIC_Map_Backend {
  status: WIC_Map_Status
}
