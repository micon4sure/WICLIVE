import { $ } from "bun"

const args = Bun.argv.slice(2)
const action = args[0]

function setEnv(vars: Record<string, string>) {
  for (const [k, v] of Object.entries(vars)) process.env[k] = v
}

const PROD_API = "https://wiclive.wicgate.org"
const LOCAL_API = "http://localhost:3243"

const envs: Record<string, { api: string, release: boolean }> = {
  development: { api: LOCAL_API, release: false },
  testing:     { api: PROD_API, release: false },
  production:  { api: PROD_API, release: true },
}

const actions: Record<string, (...args: string[]) => Promise<void>> = {
  async run(env: string) {
    const cfg = envs[env]
    if (!cfg) {
      console.log(`Usage: bun goes.ts run <${Object.keys(envs).join("|")}>`)
      process.exit(1)
    }
    setEnv({ API_URL: cfg.api, VITE_API_URL: cfg.api })
    const flag = cfg.release ? " --release" : ""
    await $`bun run tauri dev${flag}`
  },

  async build() {
    const key = (await Bun.file("src-tauri/tauri-sign.key").text()).trim()
    setEnv({ TAURI_SIGNING_PRIVATE_KEY: key, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: '', API_URL: PROD_API, VITE_API_URL: PROD_API })
    await $`bun run tauri build`
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
