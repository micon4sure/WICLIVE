import _ from "lodash"
import express from 'express';
import formidable from 'formidable';
import fs from 'fs';
import path from 'path';
import https from 'https';
import { execSync } from 'child_process';
import md5 from 'md5-file'

import keys from './keys.json'

const clearCache = () => {
  // clear cache on startup
  try {
    fs.unlinkSync('./maps/_cache.json');
    console.log('deleted cache')
  } catch (error) {
    console.log('no cache to delete')
  }
};
clearCache();

const app = express();
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  next();
});

const mapsDirectory = './maps';

// ### GET MAPS
app.get('/maps/hashes', async (req, res) => {
  console.log('GET /maps/hashes');
  fs.readdir(mapsDirectory, async (err, files) => {
    if (err) {
      console.error(err);
      res.status(500).send('Internal Server Error');
    } else {

      // read cache file if it exists
      if (fs.existsSync(mapsDirectory + '/_cache.json')) {
        console.log('returning cached map list')
        const cache = fs.readFileSync(mapsDirectory + '/_cache.json', 'utf8');
        res.json(JSON.parse(cache));
        return;
      }

      const fileHashes: { [filename: string]: string } = {};
      console.log('calculating hashes for all maps', files)
      for (let index in files) {
        const file = files[index]
        // skip files not ending in .sdf
        if (!file.endsWith('.sdf')) continue;
        const filePath = `${mapsDirectory}/${file}`;
        fileHashes[file] = (await md5(filePath)).toUpperCase();
        console.log('ADDING HASH', file, fileHashes[file])

      }
      console.log('done creating file hashes', fileHashes)

      // write result to cache file
      fs.writeFile(mapsDirectory + '/_cache.json', JSON.stringify(fileHashes), (err) => {
        if (err) {
          console.error(err);
        }
      });

      res.json(fileHashes);
    }
  });
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
    files = files.filter(file => file !== '_cache.json');
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
  res.download(filePath);
});

// ### UPLOAD MAP
const mapTempUploadDir = 'uploads';
fs.existsSync(mapTempUploadDir) || fs.mkdirSync(mapTempUploadDir, { recursive: true });
app.post('/maps/upload', async (req, res) => {
  console.log('POST /maps/upload');
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

    clearCache();
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