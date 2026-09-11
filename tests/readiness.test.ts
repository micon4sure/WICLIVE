import { beforeAll, beforeEach, describe, expect, mock, test } from 'bun:test'

let keyResult = { valid: true, detail: 'test key' }
let keyReadFails = false
let dx9ResetResult = { available: true, enabled: true }
let dx9ResetReadFails = false
let dx9ResetWriteFails = false
const invoke = mock(async (command: string, args?: Record<string, unknown>): Promise<unknown> => {
  switch (command) {
    case 'get_install_path': return 'C:\\Games\\World in Conflict'
    case 'get_autoexec_state': return [true, true]
    case 'get_dx9_reset':
      if (dx9ResetReadFails) throw new Error('executable unavailable')
      return dx9ResetResult
    case 'set_dx9_reset':
      if (dx9ResetWriteFails) throw new Error('executable locked')
      dx9ResetResult = { available: true, enabled: args?.enabled === true }
      return dx9ResetResult
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
  dx9ResetResult = { available: true, enabled: true }
  dx9ResetReadFails = false
  dx9ResetWriteFails = false
  state.wasFixed.value = false
  state.wasInstalled.value = false
  invoke.mockClear()
})

describe('DX9 reset', () => {
  test('uses the existing executable setting without changing it during readiness checks', async () => {
    for (const enabled of [true, false]) {
      dx9ResetResult = { available: true, enabled }
      await state.check()
      expect(state.dx9Reset.value).toBe(enabled)
      expect(state.dx9ResetAvailable.value).toBe(true)
      expect(state.isReady.value).toBe(true)
    }
    expect(invoke.mock.calls.some(([command]) => command === 'set_dx9_reset')).toBe(false)
  })

  test('leaves unsupported executables untouched and does not block launch', async () => {
    dx9ResetResult = { available: false, enabled: false }
    await state.check()
    expect(state.dx9ResetAvailable.value).toBe(false)
    expect(state.isReady.value).toBe(true)
    expect(await state.setDx9Reset(true)).toBe(false)
    expect(await state.setDx9Reset(false)).toBe(false)
    expect(invoke.mock.calls.some(([command]) => command === 'set_dx9_reset')).toBe(false)
  })

  test('retains the confirmed setting after a failed write and allows retry', async () => {
    await state.check()
    dx9ResetWriteFails = true
    expect(await state.setDx9Reset(false)).toBe(false)
    expect(state.dx9Reset.value).toBe(true)
    expect(state.dx9ResetError.value).toContain('executable locked')
    expect(state.dx9ResetBusy.value).toBe(false)
    dx9ResetWriteFails = false
    expect(await state.setDx9Reset(false)).toBe(true)
    expect(state.dx9Reset.value).toBe(false)
    expect(state.dx9ResetError.value).toBe('')
    expect(await state.setDx9Reset(true)).toBe(true)
    expect(state.dx9Reset.value).toBe(true)
  })

  test('disables the toggle when the executable can no longer be read', async () => {
    await state.check()
    dx9ResetReadFails = true
    await state.check()
    expect(state.dx9ResetAvailable.value).toBe(false)
    expect(state.dx9Reset.value).toBe(false)
    expect(state.dx9ResetError.value).toContain('executable unavailable')
    expect(await state.setDx9Reset(false)).toBe(false)
    expect(invoke.mock.calls.some(([command]) => command === 'set_dx9_reset')).toBe(false)
  })
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
