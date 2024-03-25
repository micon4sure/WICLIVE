import fs from 'fs';

// Get the version from the CLI parameter
const version = process.argv[2];

// read cargo.toml
const cargoTomlPath = './src-tauri/Cargo.toml';
const cargoTomlContent = fs.readFileSync(cargoTomlPath, 'utf-8');

// regex replace the version line
const updatedCargoTomlContent = cargoTomlContent.replace(/version = "(.*?)"/, `version = "${version}"`);

// write the updated content
fs.writeFileSync(cargoTomlPath, updatedCargoTomlContent, 'utf-8');

console.log(`Version updated to ${version} in Cargo.toml`);

// read tauri.conf.json
const tauriConfPath = './src-tauri/tauri.conf.json';
const tauriConfContent = fs.readFileSync(tauriConfPath, 'utf-8');

// regex replace the version line
const updatedTauriConfContent = tauriConfContent.replace(/"version": "(.*?)"/, `"version": "${version}"`);

// write the updated content
fs.writeFileSync(tauriConfPath, updatedTauriConfContent, 'utf-8');

console.log(`Version updated to ${version} in tauri.conf.json`);

// read package.json
const packageJsonPath = './package.json';
const packageJsonContent = fs.readFileSync(packageJsonPath, 'utf-8');

// regex replace the version line
const updatedPackageJsonContent = packageJsonContent.replace(/"version": "(.*?)"/, `"version": "${version}"`);

// write the updated content
fs.writeFileSync(packageJsonPath, updatedPackageJsonContent, 'utf-8');

console.log(`Version updated to ${version} in package.json`);
