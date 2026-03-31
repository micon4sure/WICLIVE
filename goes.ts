import { $ } from "bun"

const args = Bun.argv.slice(2)
const action = args[0]

const actions: Record<string, (...args: string[]) => Promise<void>> = {
  async dev() {
    await $`bun run tauri dev`
  },

  async build() {
    const key = await Bun.file("src-tauri/tauri-sign.key").text()
    process.env.TAURI_SIGNING_PRIVATE_KEY = key.trim()
    process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
    process.env.API_URL = "https://wiclive.techtile.media"
    await $`bun run tauri build`
  },

  async beta() {
    await actions.build()

    const conf = await Bun.file("src-tauri/tauri.conf.json").json()
    const version = conf.version
    const exe = `src-tauri/target/release/bundle/nsis/WIC LIVE_${version}_x64-setup.exe`

    const tag = `${version}-beta`

    // Delete existing beta release if same tag
    try { await $`gh release delete ${tag} --yes` } catch {}
    try { await $`git push origin :refs/tags/${tag}` } catch {}

    await $`gh release create ${tag} --title "Beta ${version}" --prerelease ${exe}#wiclive-setup-beta.exe`
    console.log(`Beta release created: ${tag}`)
  },
}

if (!action || !actions[action]) {
  console.log(`Usage: bun goes.ts <action>`)
  console.log(`Actions: ${Object.keys(actions).join(", ")}`)
  process.exit(1)
}

await actions[action](...args.slice(1))
