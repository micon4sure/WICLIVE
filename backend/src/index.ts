import _ from "lodash"
import express from 'express';
import formidable from 'formidable';
import fs from 'fs';
import path, { format } from 'path';
import https from 'https';
import md5 from 'md5-file'

import dotenv from 'dotenv';

import keys from '../keys.json'

const mapsDirectory = './maps';
const patchesDirectory = './patches';
const mapsDataFile = './_maps.json';
const patchesDataFile = './_patches.json';

const app = express();
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  next();
});

interface WIC_Map_Backend {
  name: string;
  hash: string;
  size: number;
  date: string;
  uploader: string;
  version: number;
}

class WIC_Map_Database_Backend {
  private maps: { [key: string]: WIC_Map_Backend } = {};

  get data() {
    return { maps: this.maps }
  }

  async init() {
    const files = fs.readdirSync(mapsDirectory);
    try {
      const data = JSON.parse(await fs.readFileSync(mapsDataFile, 'utf8'));
      this.maps = data.maps;
    } catch (error) {
      this.maps = {};
      console.log('no cache file found, building')

      const promises = _.map(files, async (file) => {
        if (!file.endsWith('.sdf')) return;
        await this.addMap(file, 'unknown');
      });
      await Promise.all(promises);
    }

    // check for maps removed on fs
    const removed = _.difference(_.keys(this.maps), files);
    _.each(removed, (map) => {
      console.log('removing map', map)
      delete this.maps[map];
    });

  }
  formatDate(date) {
    const pad = (num) => (num < 10 ? '0' + num : num);

    const year = date.getFullYear();
    const month = pad(date.getMonth() + 1); // getMonth() is zero-based
    const day = pad(date.getDate());
    const hours = pad(date.getHours());
    const minutes = pad(date.getMinutes());

    return `${year}-${month}-${day} ${hours}:${minutes}`;
  }

  async addMap(mapName, uploader) {
    const hash = await this.getMapHash(mapName);
    const size = this.getMapSize(mapName);
    if (!this.maps[mapName]) {
      this.maps[mapName] = {
        name: mapName,
        size,
        hash,
        date: this.formatDate(new Date()),
        uploader: uploader,
        version: 1
      };
    }
  }

  async uploaded(mapName, uploader) {
    if (!this.maps[mapName]) {
      await this.addMap(mapName, uploader);
      this.save();
      return;
    }
    const map = this.maps[mapName];
    map.uploader = uploader;
    map.version++;
    map.size = this.getMapSize(mapName);
    map.hash = await this.getMapHash(mapName);
    map.date = this.formatDate(new Date());
    this.save();
  }

  getMapSize(mapName) {
    return fs.statSync(`${mapsDirectory}/${mapName}`).size;
  }

  async getMapHash(mapName) {
    return (await md5(`${mapsDirectory}/${mapName}`)).toUpperCase()
  }

  async save() {
    fs.writeFileSync(mapsDataFile, JSON.stringify(this.data));
  }
}

interface WIC_Patch_Backend {
  name: string;
  uploader: string;
  hash: string;
  size: number;
  date: string;
  version: number;
}

class WIC_Patch_Database_Backend {
  private patches: { [key: string]: WIC_Patch_Backend } = {};

  get data() {
    return { patches: this.patches }
  }

  async init() {
    const files = fs.readdirSync(patchesDirectory);
    try {
      const data = JSON.parse(await fs.readFileSync(patchesDataFile, 'utf8'));
      this.patches = data.patches;
    } catch (error) {
      this.patches = {};
      console.log('no patch cache file found, building')

      const promises = _.map(files, async (file) => {
        if (!file.endsWith('.sdf')) return;
        await this.addPatch(file, 'unknown');
      });
      await Promise.all(promises);
    }

    // check for patches removed on fs
    const removed = _.difference(_.keys(this.patches), files);
    _.each(removed, (patch) => {
      console.log('removing patch', patch)
      delete this.patches[patch];
    });

  }
  formatDate(date) {
    const pad = (num) => (num < 10 ? '0' + num : num);

    const year = date.getFullYear();
    const month = pad(date.getMonth() + 1); // getMonth() is zero-based
    const day = pad(date.getDate());
    const hours = pad(date.getHours());
    const minutes = pad(date.getMinutes());

    return `${year}-${month}-${day} ${hours}:${minutes}`;
  }

  async addPatch(patchName, uploader) {
    const hash = await this.getPatchHash(patchName);
    const size = this.getPatchSize(patchName);
    if (!this.patches[patchName]) {
      this.patches[patchName] = {
        name: patchName,
        size,
        hash,
        date: this.formatDate(new Date()),
        uploader: uploader,
        version: 1
      };
    }
  }

  async uploaded(mapName, uploader) {
    if (!this.patches[mapName]) {
      await this.addPatch(mapName, uploader);
      this.save();
      return;
    }
    const map = this.patches[mapName];
    map.uploader = uploader;
    map.version++;
    map.size = this.getPatchSize(mapName);
    map.hash = await this.getPatchHash(mapName);
    map.date = this.formatDate(new Date());
    this.save();
  }


  async getPatchHash(patchName) {
    return (await md5(`${patchesDirectory}/${patchName}`)).toUpperCase()
  }

  getPatchSize(patchName) {
    return fs.statSync(`${patchesDirectory}/${patchName}`).size;
  }

  async save() {
    fs.writeFileSync(patchesDataFile, JSON.stringify(this.data));
  }

}

// init database
const mapDatabase = new WIC_Map_Database_Backend();
await mapDatabase.init();
mapDatabase.save();
console.log('loaded map cache', mapDatabase)

const patchDatabase = new WIC_Patch_Database_Backend();
await patchDatabase.init();
patchDatabase.save();
console.log('loaded patch cache', patchDatabase)


app.get('/maps/data', async (req, res) => {
  console.log('GET /maps/data');
  res.json(mapDatabase.data.maps);
});

// ### DOWNLOAD MAP
app.get('/maps/download/:filename', async (req, res) => {
  console.log(`GET /maps/download/${req.params.filename}`);
  // sanitize filename
  if (req.params.filename.includes('..') || !req.params.filename.endsWith('.sdf')) {
    res.status(400).send('Invalid filename');
    return;
  }
  const filename = req.params.filename;
  const filePath = `${mapsDirectory}/${filename}`;

  const stat = fs.statSync(filePath);

  res.header('X-Filesize', stat.size);
  res.download(filePath);
});

// ### UPLOAD MAP
const tmpUploadDir = 'uploads-tmp';
fs.existsSync(tmpUploadDir) || fs.mkdirSync(tmpUploadDir, { recursive: true });
app.post('/maps/upload', async (req, res) => {
  console.log('POST /maps/upload');

  // limit time to upload between tuesday noon and thursday noon
  const now = new Date();
  const day = now.getDay();
  const hour = now.getHours();
  if (day < 2 || day > 4 || (day === 2 && hour < 12) || (day === 4 && hour >= 12)) {
    return res.status(403).send('Uploads are only allowed between Tuesday noon and Thursday noon.');
  }

  const form = formidable();
  form.uploadDir = tmpUploadDir;
  form.keepExtensions = true;

  form.parse(req, async (err, fields, files) => {
    if (err) {
      console.log(err)
      return res.status(500).send('An error occurred during the file upload.');
    }

    const key = fields.key[0]

    if (!_.includes(Object.values(keys), key)) {
      return res.status(401).send('Invalid API key');
    }
    console.log(key, key)
    const uploader = _.findKey(keys, (value) => value === key);

    const mapName = files.file[0].originalFilename;
    if (mapName.includes('..') || !mapName.endsWith('.sdf')) {
      return res.status(400).send('Invalid filename');
    }

    console.log(`Received file: ${mapName}`);

    const tmpPath = files.file[0].filepath;
    const newPath = path.join(mapsDirectory, mapName);

    console.log(`Moving file from ${tmpPath} to ${newPath}`);
    try {
      fs.renameSync(tmpPath, newPath);
      res.send('File uploaded and moved successfully.');
    } catch (error) {
      console.error(err);
      if (err) return res.status(500).send('Error saving file.');
      return;
    }
    await mapDatabase.uploaded(mapName, uploader);
  });
})

// ### DOWNLOAD GAME AND FILES
const filesRegex = /\/files\/(.+)/;
app.get(filesRegex, async (req, res) => {
  console.log(`GET /files/${req.params[0]}`);

  if (!dotenv.config().parsed.ENV_DEVELOPMENT) {
    res.status(403).send('Forbidden');
    return;
  }

  // sanitize filename
  if (req.params[0].includes('..')) {
    res.status(400).send('Invalid filename');
    return;
  }
  const filename = req.params[0];
  const filePath = `./files/${filename}`;

  try {
    fs.accessSync(filePath, fs.constants.R_OK);
  } catch (error) {
    res.status(404).send('File not found');
    return;
  }
  const stat = fs.statSync(filePath);

  res.header('X-Filesize', stat.size);
  res.download(filePath);
});

// ### DOWNLOAD RELEASE
app.get('/wiclive/release/:version', async (req, res) => {
  // sanitize version semver including alpha/beta
  if (!req.params.version.match(/^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$/)) {
    res.status(400).send('Invalid version');
    return;
  }
  console.log(`GET / wiclive / release / ${req.params.version}`);
  const version = req.params.version;
  const release = `./ release / wiclive_${version}_x64 - setup.exe`;
  res.download(release);
})

// ### PATCH DATA
app.get('/patches/data', async (req, res) => {
  console.log('GET /patches/data');
  res.json(patchDatabase.data.patches);
});

// ### DOWNLOAD PATCH
app.get('/patches/download/:filename', async (req, res) => {
  console.log(`GET /patches/download/${req.params.filename}`);
  // sanitize filename
  if (req.params.filename.includes('..') || !req.params.filename.endsWith('.sdf')) {
    res.status(400).send('Invalid filename');
    return;
  }
  const filename = req.params.filename;
  const filePath = `${patchesDirectory}/${filename}`;

  const stat = fs.statSync(filePath);

  res.header('X-Filesize', stat.size);
  res.download(filePath);
});

// ### UPLOAD PATCH
app.post('/patches/upload', async (req, res) => {
  console.log('POST /patches/upload');

  // limit time to upload between tuesday noon and thursday noon
  const now = new Date();
  const day = now.getDay();
  const hour = now.getHours();
  if (day < 2 || day > 4 || (day === 2 && hour < 12) || (day === 4 && hour >= 12)) {
    // return res.status(403).send('Uploads are only allowed between Tuesday noon and Thursday noon.');
  }

  const form = formidable();
  form.uploadDir = tmpUploadDir;
  form.keepExtensions = true;

  form.parse(req, async (err, fields, files) => {
    if (err) {
      console.log(err)
      return res.status(500).send('An error occurred during the file upload.');
    }

    const key = fields.key[0]

    if (!_.includes(Object.values(keys), key)) {
      return res.status(401).send('Invalid API key');
    }
    console.log(key, key)
    const uploader = _.findKey(keys, (value) => value === key);

    const patchName = files.file[0].originalFilename;
    if (patchName.includes('..') || !patchName.endsWith('.sdf')) {
      return res.status(400).send('Invalid filename');
    }

    console.log(`Received file: ${patchName}`);

    const tmpPath = files.file[0].filepath;
    const newPath = path.join(patchesDirectory, patchName);

    console.log(`Moving file from ${tmpPath} to ${newPath}`);
    try {
      fs.renameSync(tmpPath, newPath);
      res.send('File uploaded and moved successfully.');
    } catch (error) {
      console.error(err);
      if (err) return res.status(500).send('Error saving file.');
      return;
    }
    await patchDatabase.uploaded(patchName, uploader);
  });
})

import ssl from './get-ssl-credentials';
const port = 3243
try {
  const server = https.createServer(ssl() as any, app);
  server.listen(port, () => {
    console.log(`SSL enabled server is running on port ${port}`);
  });
} catch (error) {
  app.listen(port, () => {
    console.log(`Server is running on port ${port}`);
  });

}