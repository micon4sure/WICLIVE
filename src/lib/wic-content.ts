export enum WIC_Content_Status {
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
  status: WIC_Content_Status
}

export interface WIC_Patch_Backend {
  name: string;
  hash: string;
  size: number;
  date: string;
  version: number;
  uploader: string;

}
export interface WIC_Patch_Frontend extends WIC_Patch_Backend {
  status: WIC_Content_Status
}
