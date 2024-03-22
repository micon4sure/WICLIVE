import _ from 'lodash';

import get_config from './get_config';

export default class WIC_Cache {
  private static singleton: WIC_Cache;
  private data: { [key: string]: WIC_Map } = {};

  private constructor() {
    const cache = localStorage.getItem('cache');
    if (cache) {
      this.data = JSON.parse(cache)
    }
  }
  public static async init() {
    const config: any = await get_config();
    if (localStorage.getItem('version') && localStorage.getItem('version') == config.VERSION)
      return;

    console.log('new version, invalidating cache')
    localStorage.clear();

    localStorage.setItem('version', config.VERSION);
  }

  public static async instance(): Promise<WIC_Cache> {
    await WIC_Cache.init();
    if (!WIC_Cache.singleton) {
      WIC_Cache.singleton = new WIC_Cache();
    }

    return WIC_Cache.singleton;
  }

  public set(key: string, value: WIC_Map): void {
    this.data[key] = value;
    localStorage.setItem('cache', JSON.stringify(this.data));
  }

  public has(key: string): boolean {
    return _.has(this.data, key);
  }

  public get(key: string): any {
    return this.data[key];
  }
}

export interface WIC_Map {
  name: string;
  hash: string;
}