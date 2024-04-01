import { $, ShellOutput } from "bun";

const args: string[] = Bun.argv;
args.shift(); // remove binary name
args.shift(); // remove script name
const action: string = args[0];

const actions = {
  async run() {
    const environment: string = args.length > 1 ? args[1] : "staging";
    console.log(`RUNNING in env ${environment}`);
    process.env.WICLIVE_ENV = environment;
    await $`bun run tauri dev`;
  },
  async runRelease() {
    const environment: string = args.length > 1 ? args[1] : "staging";
    console.log(`RUNNING RELEASE in env ${environment}`);
    process.env.WICLIVE_ENV = environment;
    await $`bun run tauri dev --release`;
  },
  async version() {
    if (args.length > 1) {
      await $`bun run ./update-version.ts ${args[1]}`;
    } else {
      console.log("Missing version argument for 'version' action.");
    }
  },
  async build() {
    const environment: string = args.length > 1 ? args[1] : "staging";
    console.log(`BUILDING in env ${environment}`);
    process.env.WICLIVE_ENV = environment;
    const privateKey = await $`cat src-tauri/tauri-sign.key`.text();
    process.env.TAURI_PRIVATE_KEY = privateKey.toString();
    process.env.TAURI_KEY_PASSWORD = "";
    if (environment === "development" || environment === "staging") {
      await $`bun run tauri build --debug -b none`;
    } else {
      await $`bun run tauri build --ci`;
    }
  },
  async act() {
    const key: string = (await $`cat src-tauri/tauri-sign.key`).text().trim()
    const token:
      string = (await $`cat .github/token`).text().trim();
    await $`./act.exe --action-offline-mode -P windows-latest=-self-hosted -j build-and-release-local -s TAURI_PRIVATE_KEY="${key}" -s GITHUB_TOKEN="${token}"`;
  }
}
actions[action]();