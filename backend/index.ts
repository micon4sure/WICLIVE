import _ from "lodash"
import express from 'express';
import formidable from 'formidable';
import fs from 'fs';
import path, { format } from 'path';
import https from 'https';
import md5 from 'md5-file'

import keys from './keys.json'

let mapsDirectory = './maps';
const cacheFile = './_cache.json';


const app = express();
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  next();
});

// read cache file
let cache = {};

const saveCache = () => {
  console.log(saveCache, cache)
  fs.writeFileSync(cacheFile, JSON.stringify(cache));
}

const formatDate = (date) => {
  const pad = (num) => (num < 10 ? '0' + num : num);

  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1); // getMonth() is zero-based
  const day = pad(date.getDate());
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());

  return `${year}-${month}-${day} ${hours}:${minutes}`;
}

try {
  cache = JSON.parse(await fs.readFileSync(cacheFile, 'utf8'));
} catch (error) {
  cache = {};

  console.log('no cache file found, building')

  const files = fs.readdirSync(mapsDirectory);
  const promises = _.map(files, async (file) => {
    if (!file.endsWith('.sdf')) return;
    const filePath = `${mapsDirectory}/${file}`;
    const hash = (await md5(filePath)).toUpperCase();
    let stats = fs.statSync(filePath);
    cache[file] = {
      name: file,
      hash,
      date: formatDate(new Date(stats.mtime)),
      uploader: 'unknown',
      version: 1
    }
  });
  await Promise.all(promises);
  saveCache();
}

app.get('/maps/data')
console.log('loaded cache', cache)

app.get('/maps/data', async (req, res) => {
  res.json(cache);
});


// ### LIST MAPS
app.get('/maps/list', async (req, res) => {
  console.log('GET /maps/list');

  const formatDate = (date) => {
    const pad = (num) => (num < 10 ? '0' + num : num);

    const year = date.getFullYear();
    const month = pad(date.getMonth() + 1); // getMonth() is zero-based
    const day = pad(date.getDate());
    const hours = pad(date.getHours());
    const minutes = pad(date.getMinutes());
    const seconds = pad(date.getSeconds());

    return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
  }

  fs.readdir(mapsDirectory, async (err, files) => {
    files = files.filter(file => file.endsWith('.sdf'));
    // get file modification dates
    const fileStats = files.map(file => {
      const stats = fs.statSync(`${mapsDirectory}/${file}`);
      return {
        name: file,
        date: formatDate(new Date(stats.mtime))
      }
    });
    // read the template file
    const templateRaw = fs.readFileSync('maplist.html', 'utf8');
    const template = _.template(templateRaw);
    const content = template({ maps: fileStats });

    if (err) {
      console.error(err);
      res.status(500).send('Internal Server Error');
    } else {
      res.send(content);
    }
  });
});

// ### DOWNLOAD MAP
app.get('/maps/download/:filename', async (req, res) => {
  console.log(`GET /maps/download/${req.params.filename}`);
  // sanitize filename
  if (req.params.filename.includes('..')) {
    res.status(400).send('Invalid filename');
    return;
  }
  const filename = req.params.filename;
  const filePath = `${mapsDirectory}/${filename}`;

  const stat = fs.statSync(filePath);

  res.download(filePath);
});

// ### UPLOAD MAP
const mapTempUploadDir = 'uploads';
fs.existsSync(mapTempUploadDir) || fs.mkdirSync(mapTempUploadDir, { recursive: true });
app.post('/maps/upload', async (req, res) => {
  console.log('POST /maps/upload');

  // limit time to upload between tuesday noon and thursday noon
  const now = new Date();
  const day = now.getDay();
  const hour = now.getHours();
  if (day < 2 || day > 4 || (day === 2 && hour < 12) || (day === 4 && hour >= 12)) {
    // return res.status(403).send('Uploads are only allowed between Tuesday noon and Thursday noon.');
  }

  const form = formidable();
  form.uploadDir = mapTempUploadDir;
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
    if (mapName.includes('..')) {
      return res.status(400).send('Invalid filename');
    }

    console.log(`Received file: ${mapName}`);

    const tmpPath = files.file[0].filepath;
    const newPath = path.join(mapsDirectory, mapName);

    console.log(`Moving file from ${tmpPath} to ${newPath}`);

    fs.rename(tmpPath, newPath, (err) => {
      if (err) return res.status(500).send('Error saving file.');
      res.send('File uploaded and moved successfully.');
    });

    cache[mapName].version++;
    cache[mapName].uploader = uploader;
    cache[mapName].date = formatDate(new Date());

    saveCache();
  });
})

import ssl from './get-ssl-credentials';
const port = 3243
try {
  const server = https.createServer(ssl() as any, app);
  server.listen(port, () => {
    console.log(`Server is running on port ${port}`);
  });
} catch (error) {
  app.listen(port, () => {
    console.log(`Server is running on port ${port}`);
  });

}