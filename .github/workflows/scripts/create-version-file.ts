import fs from 'fs';
import path from 'path';

// take cli arguments for version, repository and workspace
const version = process.argv[2];
const repository = process.argv[3];
const workspace = process.argv[4];

const url = `https://github.com/${repository}/releases/download/${version}/wiclive_${version}_x64-setup.nsis.zip`;
const signaturePath = path.join(workspace, 'src-tauri', 'target', 'release', 'bundle', 'nsis', `WIC LIVE_${version}_x64-setup.nsis.zip.sig`);

// Read the signature file
fs.readFile(signaturePath, 'utf8', (err, signature) => {
  if (err) {
    console.error('Error reading the signature file:', err);
    return;
  }

  // Generate JSON
  const json = {
    version: version,
    platforms: {
      "windows-x86_64": {
        signature: signature,
        url: url
      }
    }
  };

  // Write version.json
  fs.writeFile('version.json', JSON.stringify(json, null, 2), 'utf8', (err) => {
    if (err) {
      console.error('Error writing version.json:', err);
      return;
    }
    console.log('version.json generated successfully.');
  });
});
