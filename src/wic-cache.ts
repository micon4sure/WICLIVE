import _ from 'lodash';

export default class WIC_Cache {
  private static singleton: WIC_Cache;
  private data: { [key: string]: WIC_Map } = {};

  private constructor() {
    localStorage.setItem('cache', JSON.stringify(contents));
    const cache = localStorage.getItem('cache');
    if (cache) {
      this.data = JSON.parse(cache)
    }
  }

  public static instance(): WIC_Cache {
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