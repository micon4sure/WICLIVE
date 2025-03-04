import { $ } from "bun";
import _ from 'lodash'
import semver from 'semver'
import { rename } from 'fs/promises';
const args: string[] = Bun.argv;
args.shift(); // remove binary name
args.shift(); // remove script name
const action: string = args[0];

const actions = {
  async run(environment: string = "development") {
    console.log(`RUNNING in env ${environment}`);
    process.env.WICLIVE_ENV = environment;
    await $`bun run tauri dev`;
  },
  async runRelease(environment: string = "development") {
    console.log(`RUNNING RELEASE in env ${environment}`);
    process.env.WICLIVE_ENV = environment;
    await $`bun run tauri dev --release`;
  },
  async increment_version() {
    // read version from package.json
    const packageRaw: string = await $`cat package.json`.text()
    const packageJson = JSON.parse(packageRaw);
    const version = packageJson.version;

    // if is prerelease
    let incremented
    if (semver.prerelease(version)) {
      console.log('version is prerelease');
      incremented = semver.inc(version, 'prerelease', 'beta');
      console.log('set version to', incremented)
    } else {
      incremented = semver.inc(version, 'patch');
    }
    console.log('incremented version from', version, 'to', incremented)
    actions.set_version(incremented)

    console.log('set version to', incremented)
    return incremented
  },
  async set_version(version: string) {
    if (version) {
      await $`bun run ./update-version.ts ${version}`;
    } else {
      throw new Error("Missing version argument for 'version' action.");
    }
  },
  async build(environment: string = "testing", incrementVersion: boolean = true) {
    if (incrementVersion)
      await actions.increment_version()

    console.log(`BUILDING for environment ${environment}`);
    process.env.WICLIVE_ENV = environment;
    const privateKey = await $`cat src-tauri/tauri-sign.key`.text();
    process.env.TAURI_PRIVATE_KEY = privateKey.toString();
    process.env.TAURI_KEY_PASSWORD = "";
    switch (environment) {
      case "testing":
      case "staging":
        try {
          await $`bun tauri build --debug --ci -b nsis`;
        } catch (error) {
          console.error('BUILD ERROR:', error.stderr.toString())
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
  },

  async release_beta() {
    const version = await actions.increment_version()

    // Remove any existing debug assets from GitHub releases
    console.log("Finding existing debug release assets");
    const token = await $`cat .github/token`.text();
    const releases = await $`curl -H "Authorization: token ${token}" -s https://api.github.com/repos/micon4sure/WICLIVE/releases`.json();
    for (let release of releases) {
      console.log("release:", release.tag_name);
      const assets = await $`curl -H "Authorization: token ${token}" -s https://api.github.com/repos/micon4sure/WICLIVE/releases/${release.id}/assets`.json();
      for (let asset of assets) {
        if (asset.name === 'wiclive_x64-setup-debug.exe' || asset.name === 'wiclive-setup-beta.exe') {
          await $`curl -X DELETE -H "Authorization: token ${token}" -s https://api.github.com/repos/micon4sure/WICLIVE/releases/assets/${asset.id}`;
          console.log("deleted:", release.tag_name + "/" + asset.name);
        }
      }
      // Only loop until release "2.0.0" (debug assets did not exist before then)
      if (release.tag_name === '2.0.0') break;
    }
    const latestRelease = releases[0];

    // Set common Tauri signing variables
    const privateKey = await $`cat src-tauri/tauri-sign.key`.text();
    process.env.TAURI_PRIVATE_KEY = privateKey.toString();
    process.env.TAURI_KEY_PASSWORD = "";

    console.log(`BUILDING STAGING (debug) for version ${version}`);
    await actions.build("staging", false);
    // Rename staging artifact to the expected filename
    const stagingSourcePath = `./src-tauri/target/debug/bundle/nsis/WIC LIVE_${version}_x64-setup.exe`;
    const stagingDestPath = "./wiclive_x64-setup-debug.exe";
    console.log(`Renaming ${stagingSourcePath} -> ${stagingDestPath}`);
    await rename(stagingSourcePath, stagingDestPath);

    // Build for production (release build)
    console.log(`BUILDING PRODUCTION (release) for version ${version}`);
    await actions.build("production", false);
    // Rename production artifact to the expected filename
    const productionSourcePath = `./src-tauri/target/release/bundle/nsis/WIC LIVE_${version}_x64-setup.exe`;
    const productionDestPath = "./wiclive-setup-beta.exe";
    console.log(`Renaming ${productionSourcePath} -> ${productionDestPath}`);
    await rename(productionSourcePath, productionDestPath);

    // Upload staging asset (debug build)
    console.log("Uploading staging asset as wiclive_x64-setup-debug.exe");
    let stagingUpload = await $`
      curl -X POST -H "Authorization: token ${token}" \
        -H "Content-Type: application/octet-stream" \
        --data-binary @"${stagingDestPath}" \
        "https://uploads.github.com/repos/micon4sure/WICLIVE/releases/${latestRelease.id}/assets?name=wiclive_x64-setup-debug.exe"
    `.json();
    console.log("Staging asset uploaded – available at", stagingUpload.browser_download_url);

    // Upload production asset (release build)
    console.log("Uploading production asset as wiclive-setup-beta.exe");
    let productionUpload = await $`
      curl -X POST -H "Authorization: token ${token}" \
        -H "Content-Type: application/octet-stream" \
        --data-binary @"${productionDestPath}" \
        "https://uploads.github.com/repos/micon4sure/WICLIVE/releases/${latestRelease.id}/assets?name=wiclive-setup-beta.exe"
    `.json();
    console.log("Production asset uploaded – available at", productionUpload.browser_download_url);
  }

}
await actions[action](args.length > 1 ? args[1] : undefined);