const express = require('express');
const app = express();
const multer = require('multer');
const vosk = require('vosk');
const AudioBufferToWav = require('audiobuffer-to-wav');
const randomText = require('./data');
const fs = require('fs');
const bodyParser = require('body-parser');
const config = require('./config');
const { createProxyMiddleware } = require('http-proxy-middleware');

const upload = multer();

const pDict = {};

const pronounciationFile = fs.readFileSync('pronounciation.txt', { encoding: 'utf8', flag: 'r' });
for (let line of pronounciationFile.split("\n")) {
  const [word, pron] = line.split("  ");
  pDict[word] = pron;
}

function getPronounciation(word) {
  return pDict[word.toUpperCase()] ?? "";
}

vosk.setLogLevel(2);

const model = new vosk.Model("./model");

app.use(bodyParser.urlencoded({ extended: true }));
app.use(express.json());
app.use(express.static('public'));

app.post("/api/pronounce", (req, res) => {
  const words = req.body.words ?? [];
  res.json({ 
    result: words.map(word => word.replace(/[^a-zA-Z0-9]/g, "").toUpperCase())
                 .map(word => getPronounciation(word))
  });
});

app.get("/api/random", (req, res) => {
  const items = randomText.data;
  const p = items[Math.floor(Math.random()*items.length)];
  const selected = p.sentence;
  const result = { 
    text: selected,
    pronounce: selected.toUpperCase().split(" ").map(word => getPronounciation(word.replace(/[^a-zA-Z0-9]/g, ""))).join(" ￤ ")  
  };
  res.json(result);
});

app.post('/api/process', upload.single('audio'), async (req, res) => {
  const oggBuffer = req.file.buffer;
  
  const rec = new vosk.Recognizer({
    model: model,
    sampleRate: 48000
  });
  rec.setMaxAlternatives(0);
  rec.setWords(true);

  const done = rec.acceptWaveform(oggBuffer);  
  res.json(rec.finalResult(rec));
  rec.free();
});


app.use(
  '/js/info.js', 
  createProxyMiddleware({ 
    target: config.TRACKER_DOMAIN, 
    changeOrigin: true,
    pathRewrite: { 
      '^/js/info.js': config.TRACKER_SCRIPT 
    } 
  })
);

app.use(
  '/api/event',
  createProxyMiddleware({
    target: config.TRACKER_DOMAIN,
    changeOrigin: true
  }),
);

const PORT = process.env.PORT || 3000;
app.listen(PORT, () => {
  console.log('server is running...');
});

