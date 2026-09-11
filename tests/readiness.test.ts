import { beforeAll, beforeEach, describe, expect, mock, test } from 'bun:test'

let keyResult = { valid: true, detail: 'test key' }
let keyReadFails = false
const invoke = mock(async (command: string): Promise<unknown> => {
  switch (command) {
    case 'get_install_path': return 'C:\\Games\\World in Conflict'
    case 'get_autoexec_state': return [true, true]
    case 'get_game_version': return { major: 1, minor: 0, patch: 1, build: 1 }
    case 'check_cd_key':
      if (keyReadFails) throw new Error('registry unavailable')
      return keyResult
    case 'is_compatibility_proxy': return false
    case 'get_proxy_version':
    case 'get_latest_proxy_version': return '1.0'
    case 'check_vcredist':
    case 'check_dx9':
    case 'get_laa_flag':
    case 'get_skip_launcher_flag':
    case 'check_proxy': return true
    default: throw new Error(`Unexpected command: ${command}`)
  }
})

mock.module('@tauri-apps/api/core', () => ({ invoke }))
let state: ReturnType<typeof import('../src/composables/useGameState').useGameState>

beforeAll(async () => {
  state = (await import('../src/composables/useGameState')).useGameState()
  await state.check()
})

beforeEach(() => {
  keyResult = { valid: true, detail: 'test key' }
  keyReadFails = false
  state.wasFixed.value = false
  state.wasInstalled.value = false
  invoke.mockClear()
})

describe('CD-key readiness', () => {
  test('allows launch when the native check accepts the key and edition', async () => {
    await state.check()
    expect(state.readinessActions.value.cdkey).toEqual({ need: true, has: true, detail: 'test key' })
    expect(state.needFix.value).toBe(false)
    expect(state.isReady.value).toBe(true)
    expect(invoke.mock.calls.some(([command]) => command === 'check_cd_key')).toBe(true)
    expect(invoke.mock.calls.some(([command]) => command === 'get_cd_key')).toBe(false)
  })

  for (const detail of ['Not set', 'Invalid CD key', 'Wrong game edition']) {
    test(`requires repair for ${detail.toLowerCase()}`, async () => {
      keyResult = { valid: false, detail }
      await state.check()
      expect(state.readinessActions.value.cdkey).toEqual({ need: true, has: false, detail })
      expect(state.needFix.value).toBe(true)
      expect(state.isReady.value).toBe(false)
    })
  }

  test('an earlier successful repair cannot bypass a later key mismatch', async () => {
    state.wasFixed.value = true
    state.wasInstalled.value = true
    keyResult = { valid: false, detail: 'Wrong game edition' }
    await state.check()
    expect(state.isReady.value).toBe(false)

    keyResult = { valid: true, detail: 'replacement key' }
    await state.check()
    expect(state.readinessActions.value.cdkey.has).toBe(true)
    expect(state.needFix.value).toBe(false)
    expect(state.isReady.value).toBe(true)
  })

  test('a failed key check blocks launch even after a previous repair', async () => {
    state.wasFixed.value = true
    keyReadFails = true
    await state.check()
    expect(state.readinessActions.value.cdkey).toEqual({ need: true, has: false, detail: 'Error' })
    expect(state.needFix.value).toBe(true)
    expect(state.isReady.value).toBe(false)
  })
})
