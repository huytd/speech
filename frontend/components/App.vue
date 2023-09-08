<script setup>
import { ref, reactive } from 'vue';

let stream = null;
let recorder = null;
let audioContext = null;
let input = null;

async function getRandomQuote() {
  const res = await fetch("/api/example");
  return await res.json();
}

const isRecording = ref(false);
const isAnalyzing = ref(false);
const words = ref([]);
const currentAudioBlob = ref(null);
const randomQuote = reactive({
  text: "",
  pronounce: ""
});
const score = ref(0);
const lookupWord = ref("");
const lookupWordResult = ref("");
const improvementList = ref([]);

(async () => {
  let { text, pronounce } = await getRandomQuote();
  randomQuote.text = text;
  randomQuote.pronounce = pronounce.join(" ¦ ");
})();

async function startRecord() {
  stream = await navigator.mediaDevices.getUserMedia({
    video: false,
    audio: {
      channelCount: 1,
      sampleRate: 48000
    }
  });

  audioContext = new (window.AudioContext || window.webkitAudioContext)();
  input = audioContext.createMediaStreamSource(stream);

  recorder = new WebAudioRecorder(input, {
    workerDir: "/lib/", // location of the WebAudioRecorderWorker.js file
    encoding: "wav", // record in WAV format
    numChannels: 1, // stereo recording
  });

  /**
   * @param recorder
   * @param blob {Blob}
   * @returns {Promise<void>}
   */
  recorder.onComplete = async function(recorder, blob) {
    recorder = null;
    stream = null;

    currentAudioBlob.value = blob;
    isAnalyzing.value = true;

    const formData = new FormData();
    formData.append('audio', blob);

    const res = await fetch('/api/analyze', {
      method: 'POST',
      body: formData
    });

    const result = await res.json();
    const allWords = result.words ?? [];
    // TODO: error handling
    words.value = allWords;
    score.value = allWords.reduce((total, word) => total + word.score, 0) * 100 / allWords.length;
    isAnalyzing.value = false;

    const poorWords = [...new Set(allWords.filter(word => word.score < 0.8).map(word => word.text))];
    if (poorWords.length) {
      const res = await fetch("/api/pronounce", {
        method: "POST",
        headers: {
          "content-type": "application/json"
        },
        body: JSON.stringify(poorWords)
      });
      const response = await res.json();
      improvementList.value = response.map((pron, i) => ({
        word: poorWords[i],
        pronounce: pron
      }));
    }
  }

  recorder.startRecording();
  isRecording.value = true;
}

async function stopRecord() {
  stream.getAudioTracks()[0].stop();
  recorder.finishRecording();
  isRecording.value = false;
}

function playWord(word) {
  const audioUrl = window.URL.createObjectURL(currentAudioBlob.value);
  const audio = new Audio(audioUrl + `#t=${word.start},${word.end}`);
  audio.play();
}

function playFullAudio() {
  const audioUrl = window.URL.createObjectURL(currentAudioBlob.value);
  const audio = new Audio(audioUrl);
  audio.play();
}

async function startLookup() {
  const res = await fetch("/api/pronounce", {
    method: "POST",
    headers: {
      "content-type": "application/json"
    },
    body: JSON.stringify([lookupWord.value])
  });
  const response = await res.json();
  lookupWordResult.value = response[0];
}
</script>

<template>
  <div class="app-section">
    <div class="content">
      <div class="content-box" v-if="isAnalyzing">Analyzing...</div>
      <div class="content-box" v-if="!isRecording && !isAnalyzing && words.length === 0">
        Click the "Start Record" button and say something, then we will analyze your speech.<br/>
        If you don't know what to say, try to read the random quote below.
      </div>
      <div class="content-box" v-if="isRecording">
        When you are done, click "Stop Recording"
      </div>
      <div v-if="!isAnalyzing && words.length > 0">
        <h2>your result: <span v-if="!isAnalyzing && score !== 0">{{score.toFixed(2)}}%</span></h2>
        <div class="content-box" id="result">
          <button v-for="word in words"
                  :class="{
                  good: word.score > 0.85,
                  average: word.score >= 0.75 && word.score <= 0.85,
                  poor: word.score > 0.5 && word.score <= 0.75,
                  bad: word.score <= 0.5
                }"
                  class="word"
                  @click="playWord(word)">
            {{ word.text }}
            <div class="score">{{ (word.score * 100).toFixed(2) }}%</div>
          </button>
        </div>
        <div style="margin-top: 8px">You can click on each word to hear what you said.</div>
      </div>
      <div class="controllers">
        <button class="btn" v-if="!isRecording && !isAnalyzing" @click="startRecord">
          <svg xmlns="http://www.w3.org/2000/svg" width="1.0em" height="1.0em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5a3 3 0 0 1 3-3h0a3 3 0 0 1 3 3v5a3 3 0 0 1-3 3h0a3 3 0 0 1-3-3z"/><path d="M5 10a7 7 0 0 0 14 0M8 21h8m-4-4v4"/></g></svg>
          Start Record
        </button>
        <button class="btn" v-if="isRecording" @click="stopRecord">
          <svg xmlns="http://www.w3.org/2000/svg" width="1.0em" height="1.0em" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="m3 3l18 18M9 5a3 3 0 0 1 6 0v5a3 3 0 0 1-.13.874m-2 2A3 3 0 0 1 9 10.002v-1"/><path d="M5 10a7 7 0 0 0 10.846 5.85m2-2A6.967 6.967 0 0 0 18.998 10M8 21h8m-4-4v4"/></g></svg>
          Stop Record
        </button>
        <button class="btn" v-if="!isAnalyzing && currentAudioBlob !== null" @click="playFullAudio">
          <svg xmlns="http://www.w3.org/2000/svg" width="1.0em" height="1.0em" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16l13-8z"/></svg>
          Listen
        </button>
      </div>
    </div>
  </div>

  <!-- ATTENTION SECTIOn -->
  <div class="content-section highlight" v-if="improvementList.length > 0">
    <div class="content">
      <h2>your pronunciation</h2>
      <p>Here are the list of words that can be improved:</p>
      <div style="margin-bottom: 6px" v-for="w in improvementList">
        <div><b>{{w.word}}</b></div>
        <div>{{w.pronounce}}</div>
      </div>
    </div>
  </div>

  <!-- EXTRA REPORTS -->
  <div class="content-section">
    <div class="content">
      <h2>example</h2>
      <p>“{{ randomQuote.text }}”</p>
      <h2>how to pronounce it?</h2>
      <p>{{ randomQuote.pronounce }}</p>
      <h2>what the heck is AY1 or IH0?</h2>
      <p>The pronunciation shown here is based on <a href="http://www.speech.cs.cmu.edu/cgi-bin/cmudict" rel="noopener noreferrer" target="_blank">The CMU Pronouncing Dictionary</a>, which follows the rule defined in this dictionary.</p>
      <p>The number after a sound is used to mark the lexical stress of that sound.</p>
      <p></p>
      <p><b>0</b> – no stress<br/><b>1</b> – primary stress<br/><b>2</b> – secondary stress</p>
      <h2>pronunciation lookup</h2>
      <p>Which word do you want to lookup?</p>
      <div class="word-lookup">
        <input type="text" v-model="lookupWord" />
        <button class="btn inline" @click="startLookup()">Look up</button>
        <div class="result">{{ lookupWordResult }}</div>
      </div>
    </div>
  </div>
</template>