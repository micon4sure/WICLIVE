import _ from 'lodash'

export default class Bus {
  handlers: Object;

  constructor() {
    this.handlers = {}
  }

  on(id, callback) {
    if (this.handlers[id] === undefined) {
      this.handlers[id] = []
    }
    this.handlers[id].push(callback)
    return { id, index: this.handlers[id].length - 1 };
  }
  off(params) {
    this.handlers[params.id][params.index] = null
  }
  emit(id, params = {}) {
    _.each(this.handlers[id], handler => {
      if (handler === null) return;
      handler(params)
    })
  }
}