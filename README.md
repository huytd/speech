**Speech** is an online voice recorder that checks and helps you improve your pronunciation.

Try it online at: https://speech.sege.dev

![](screenshot.png)

## Why did I build this?

As a non-native English speaker, I struggle with my pronunciation a lot. This is a tool I wished I had but could not find it anywhere, so I decided to build it myself.

Or maybe it's just a good excuse for yet another side project.

## How it works?

Under the hood, **Speech** uses [Vosk](https://alphacephei.com/vosk/) – the speech recognition toolkit, to check your voice recording and figure out what you are trying to say.

The audio recording is done using the [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API), and encoded into WAV format using the [WebAudioRecorder.js](https://github.com/higuma/web-audio-recorder-js) library.

## How to run it locally?

First, you need to download a Vosk model at https://alphacephei.com/vosk/models and extract it to the `model` folder in the source directory.

Also, you will need to have `libvosk.so` or `libvosk.dylib` in your computer's library path. You can download it from the [vosk-api](https://github.com/alphacep/vosk-api/releases/tag/v0.3.42) repository, and copy the library file to somewhere like `/usr/local/lib`.

Then you are ready to run the project:

```
$ cargo run
```

The frontend will be built automatically using [Parcel](https://parceljs.org/), and the server will be started at `http://localhost:3000`.

## Acknowledge

The speech score provided by this tool is based on the confidence score of each word provided by Vosk's model.

The pronunciation data is provided by [The CMU Pronouncing Dictionary](http://www.speech.cs.cmu.edu/cgi-bin/cmudict).

The speech examples are collected from the [Random Sentence Generator](https://randomwordgenerator.com/sentence.php) website.