import { reactive } from "vue";
import _ from "lodash";
import { WIC_Map_Backend, WIC_Map_Frontend, WIC_Map_Status } from "./wic-map";

export default class WIC_Cache {

  private maps: { [key: string]: WIC_Map_Frontend } = reactive({});

  get data() {
    return this.maps
  }

  init() {
    let stored = localStorage.getItem('wic-cache')
    if (!stored) {
      // remove all maps
      _.each(this.maps, (map, name) => {
        delete this.maps[name]
      })
      this.save()
      return;
    }
    const parsed = JSON.parse(stored)
    _.each(parsed, (map: WIC_Map_Frontend, name) => {
      this.maps[name] = map
    })

    this.save()
  }

  add(map: WIC_Map_Backend, status: WIC_Map_Status) {
    this.maps[map.name] = { name: map.name, hash: '', status, uploader: map.uploader } as WIC_Map_Frontend
    this.save()
  }

  has(name: string) {
    return this.maps[name] != undefined
  }
  get(name: string) {
    return this.maps[name]
  }
  remove(name: string) {
    delete this.maps[name]
    this.save()
  }

  save() {
    localStorage.setItem('wic-cache', JSON.stringify(this.maps))
  }
}
