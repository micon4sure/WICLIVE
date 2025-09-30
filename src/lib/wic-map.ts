export enum WIC_Map_Status {
  MISSING = 'missing',
  OUTDATED = 'outdated',
  PENDING = 'pending',
  LOADING = 'loading',
  CURRENT = 'current',
  UNKNOWN = 'unknown'
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
