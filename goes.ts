import { $ } from "bun"

const args = Bun.argv.slice(2)
const action = args[0]

async function pwsh(cmd: string) {
  const winDir = (await $`wslpath -w .`.text()).trim()
  await $`powershell.exe -NoProfile -Command ${`cd '${winDir}'; ${cmd}`}`
}

const actions: Record<string, (...args: string[]) => Promise<void>> = {
  async dev() {
    await pwsh("bun run tauri dev")
  },

  async build() {
    const key = (await Bun.file("src-tauri/tauri-sign.key").text()).trim()
    await pwsh(`$env:TAURI_SIGNING_PRIVATE_KEY='${key}'; $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=''; $env:API_URL='https://wiclive.techtile.media'; bun run tauri build`)
  },

  async beta() {
    await actions.build()
    const conf = await Bun.file("src-tauri/tauri.conf.json").json()
    const version = conf.version
    const exe = `src-tauri/target/release/bundle/nsis/WIC LIVE_${version}_x64-setup.exe`

    try { await $`gh release delete ${version} --yes` } catch {}
    try { await $`git push origin :refs/tags/${version}` } catch {}

    await $`gh release create ${version} --title ${version} --prerelease ${exe}#wiclive-setup-beta.exe`
    console.log(`Beta release created: ${version}`)
  },
}

if (!action || !actions[action]) {
  console.log(`Usage: bun goes.ts <action>`)
  console.log(`Actions: ${Object.keys(actions).join(", ")}`)
  process.exit(1)
}

await actions[action](...args.slice(1))
