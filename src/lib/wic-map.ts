export enum WIC_Map_Status {
  MISSING = 'missing',
  OUTDATED = 'outdated',
  PENDING = 'pending',
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
export interface WIC_Map_Frontend {
  name: string;
  hash: string;
  uploader: string;
  status: WIC_Map_Status
}
export interface WIC_Map_Display extends WIC_Map_Backend {
  status: WIC_Map_Status
}
