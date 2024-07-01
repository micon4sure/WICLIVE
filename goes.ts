import { $ } from "bun";
import _ from 'lodash'
import semver from 'semver'

const args: string[] = Bun.argv;
args.shift(); // remove binary name
args.shift(); // remove script name
const action: string = args[0];

const actions = {
  async run() {
    const environment: string = args.length > 1 ? args[1] : "development";
    console.log(`RUNNING in env ${environment}`);
    process.env.WICLIVE_ENV = environment;
    await $`bun run tauri dev`;
  },
  async runRelease() {
    const environment: string = args.length > 1 ? args[1] : "testing";
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
    // read version from package.json
    const packageRaw: string = await $`cat package.json`.text()
    const packageJson = JSON.parse(packageRaw);
    const version = packageJson.version;

    // if is prerelease
    let incremented
    if (semver.prerelease(version)) {
      console.log('version is prerelease');
      incremented = semver.inc(version, 'prerelease', 'beta');
      await $`bun run ./update-version.ts ${incremented}`;
      console.log('set version to', incremented)
    } else {
      incremented = semver.inc(version, 'patch');
      await $`bun run ./update-version.ts ${incremented}`;
    }

    console.log('set version to', incremented)
    const environment: string = args.length > 1 ? args[1] : "testing";
    console.log(`BUILDING for environment ${environment}`);
    process.env.WICLIVE_ENV = environment;
    const privateKey = await $`cat src-tauri/tauri-sign.key`.text();
    process.env.TAURI_PRIVATE_KEY = privateKey.toString();
    process.env.TAURI_KEY_PASSWORD = "";
    switch (environment) {
      case "testing":
        try {
          await $`bun tauri build --debug --ci -b nsis`;
        } catch (error) {
          console.error('BUILD ERROR:', error)
          return
        }
        break;
      case "production":
        await $`bun tauri build --ci`;
        return;
      default:
        console.log("Invalid environment for 'build' action.");
        return;
    }

    // remove first found debug asset
    console.log('finding debug release assets')
    const token = await $`cat .github/token`.text();
    const releases = await $`curl -H "Authorization: token ${token}" -s https://api.github.com/repos/micon4sure/WICLIVE/releases`.json();
    releaseLoop:
    for (let release of releases) {
      console.log('release:', release.tag_name)
      const assets = await $`curl -H "Authorization: token ${token}" -s https://api.github.com/repos/micon4sure/WICLIVE/releases/${release.id}/assets`.json();
      for (let asset of assets) {
        if (asset.name == 'wiclive_x64-setup-debug.exe') {
          await $`curl -X DELETE -H "Authorization: token ${token}" -s https://api.github.com/repos/micon4sure/WICLIVE/releases/assets/${asset.id}`;

          console.log('deleted:', release.tag_name, + '/' + asset.name)
          break releaseLoop;
        }
      }

      // only loop until 2.0.0, no debug assets before then
      if (release.tag_name == '2.0.0')
        break releaseLoop;
    }

    const latestRelease = releases[0];

    // upload debug release asset
    console.log('uploading debug release asset')
    const path = `./src-tauri/target/debug/bundle/nsis/WIC LIVE_${version}_x64-setup.exe`
    const uploaded = await $`
      curl -X POST -H "Authorization: token ${token}" \
        -H "Content-Type: application/octet-stream)" \
        --data-binary @"${path}" \
        "https://uploads.github.com/repos/micon4sure/WICLIVE/releases/${latestRelease.id}/assets?name=wiclive_x64-setup-debug.exe"
    `.json();
    console.log(uploaded)
    console.log("BUILD SUCCESSFUL");
  },
  async act() {
    const key: string = (await $`cat src-tauri/tauri-sign.key`).text().trim()
    const token:
      string = (await $`cat .github/token`).text().trim();
    await $`./act.exe --action-offline-mode -P windows-latest=-self-hosted -j build-and-release-local -s TAURI_PRIVATE_KEY="${key}" -s GITHUB_TOKEN="${token}"`;
  },
}
await actions[action]();