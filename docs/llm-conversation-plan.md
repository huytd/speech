# LLM Conversation + Pronunciation Checker — Implementation Plan

## Overview

Convert the existing speech pronunciation checker into a conversational AI app using **OpenAI Realtime API (speech-to-speech)**, while keeping the existing **Vosk-based pronunciation analysis** to provide real-time visual feedback on the user's spoken words.

---

## System Architecture: Before vs After

### Current System (Before)

```
. ┌───────────────────────────────────────┐
. │          Browser (Vue 3 SPA)          │
. │                                       │
. │  Mic ──WAV──► WebAudioRecorder        │
. │                  │                    │
. │                  ▼                    │
. │           POST /api/analyze           │
. │                  │                    │
. │       ┌──────────┘                    │
. │       ▼                               │
. │  Pronunciation ◄── word confidences   │
. │       │                               │
. │       │ click word                    │
. │       ▼                               │
. │  Play word segment                    │
. └───────┼───────────────────────────────┘
.         │ HTTP
.         ▼
. ┌───────────────────────────────────────┐
. │        Rust / Axum Server             │
. │                                       │
. │   ┌──────────┐   ┌────────────────┐   │
. │   │ Vosk ASR │◄──│  /api/analyze  │   │
. │   │          │──►│  (POST)        │   │
. │   │ word +   │   └────────────────┘   │
. │   │ conf +   │   ┌────────────────┐   │
. │   │ timing   │   │ /api/pronounce │   │
. │   │          │   │ (POST)         │   │
. │   └──────────┘   └───────┬────────┘   │
. │        ▲                 │            │
. │        │                 │            │
. │   ┌────┴─────┐    ┌──────▼───────┐    │
. │   │ CMUdict  │◄───│   examples   │    │
. │   │(ARPABET) │    │    .txt      │    │
. │   └──────────┘    └──────────────┘    │
. └───────────────────────────────────────┘
```

### New System (After)

```
. ┌──────────────────────────────────────────────────┐
. │                Browser (Vue 3 SPA)               │
. │                                                  │
. │  ┌──────────────────┐    ┌───────────────────┐   │
. │  │ Conversation Tab │    │   Practice Tab    │   │
. │  │                  │    │ (keep existing)   │   │
. │  │  ┌──────────┐    │    └───────────────────┘   │
. │  │  │WebSocket │    │                            │
. │  │  │ Client   │────┼──── audio chunks           │
. │  │  └────┬─────┘    │      │                     │
. │  │       │          │      │                     │
. │  │  ┌────▼─────┐    │      │                     │
. │  │  │Conversa- │    │      │                     │
. │  │  │tion UI   │    │      │                     │
. │  │  └────┬─────┘    │      │                     │
. │  │       │          │      │                     │
. │  │  ┌────▼─────┐    │      │                     │
. │  │  │Pronun-   │    │      │                     │
. │  │  │ciation   │    │      │                     │
. │  │  │Overlay   │    │      │                     │
. │  │  └────┬─────┘    │      │                     │
. │  │       │ click    │      │                     │
. │  │  ┌────▼─────┐    │      │                     │
. │  │  │Play word │    │      │                     │
. │  │  │ segment  │    │      │                     │
. │  │  └──────────┘    │      │                     │
. │  └──────────────────┘      │                     │
. │                            │                     │
. └────────────────────────────┼─────────────────────┘
.                              │ WebSocket
.                              ▼
. ┌──────────────────────────────────────────────────┐
. │              Rust / Axum Server                  │
. │                                                  │
. │  ┌───────────────────┐      ┌────────────────┐   │
. │  │ /ws/conversation  │─────►│  Audio Buffer  │   │
. │  │ (WebSocket)       │      └───────┬────────┘   │
. │  │                   │              │ on commit  │
. │  │         ┌─────────┘              ▼            │
. │  │         │                ┌──────────────┐     │
. │  │         ▼                │  Vosk ASR    │     │
. │  │  ┌──────────────┐        │              │     │
. │  │  │ OpenAI Proxy │        │ word confs   │     │
. │  │  └──────┬───────┘        └──────┬───────┘     │
. │  │         │                       │             │
. │  │    ┌────▼─────┐                 │             │
. │  │    │  OpenAI  │                 │             │
. │  │    │ Realtime │                 │             │
. │  │    │   API    │                 │             │
. │  │    └──────────┘                 │             │
. │  │                                 │             │
. │  │  ┌──────────┐  ┌──────────┐     │             │
. │  │  │ CMUdict  │  │examples  │     │             │
. │  │  │ (kept)   │  │ .txt     │     │             │
. │  │  └──────────┘  └──────────┘     │             │
. │  │                                 │             │
. │  │     Keep existing REST endpoints:             │
. │  │  /api/analyze  /api/pronounce  /api/example   │
. └──────────────────────────────────────────────────┘
```

### Data Flow (New System)

```
User (Browser)    Backend WS (/ws/conversation)    Vosk ASR    OpenAI Realtime API
      │                      │                      │                │
      │──── connect ────────►│                      │                │
      │                      │──── connect (key) ───┼───────────────►│
      │                      │◄── session.created ──┼────────────────│
      │                      │                      │                │
      │ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ Every utterance ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│
      │                      │                      │                │
      │── audio_chunks ─────►│                      │                │
      │                      │── buffer audio ─────►│                │
      │                      │── audio_chunks ──────┼───────────────►│
      │── audio_commit ─────►│                      │                │
      │                      │                      │                │
      │  ┌────────────────────────────────────────────────────────┐  │
      │  │              PARALLEL (two concurrent paths)           │  │
      │  ├──────────────────────────┬─────────────────────────────┤  │
      │  │  OpenAI Response         │  Vosk Pronunciation         │  │
      │  │                          │                             │  │
      │  │◄── transcript ───────────┤◄──── run ASR ───────────────│  │
      │  │◄── audio delta (stream)──┤◄──── SpeechAnalyzeResult ───│  │
      │  │◄── audio done ───────────┤                             │  │
      │  │◄── text done ────────────┤                             │  │
      │  │                          │                             │  │
      │  │◄── llm_response ─────────┤                             │  │
      │  │◄── pronunciation ────────┤                             │  │
      │  └──────────────────────────┴─────────────────────────────┘  │
      │                      │                      │                │
      │  render chat bubble with colored word tiles │                │
      │  show pronunciation overlay                 │                │
      │                      │                      │                │
      └── disconnect ────────►── disconnect ────────────────────────►│
```

---

## Phases

### Phase 1 — Backend: OpenAI Realtime API Proxy

**Goal:** Establish WebSocket infrastructure to connect frontend ↔ backend ↔ OpenAI.

#### Files to modify/create:

| File | Action | Description |
|------|--------|-------------|
| `Cargo.toml` | Modify | Add `axum` `ws` feature, add `tokio-tungstenite`, `base64`, `futures-util`, `tokio-stream` |
| `src/openai.rs` | **Create** | OpenAI Realtime API WebSocket client |
| `src/conversation.rs` | **Create** | Conversation manager — routes messages between frontend WS, OpenAI WS, and Vosk |
| `src/api.rs` | Modify | Add WebSocket route `/ws/conversation` |
| `src/main.rs` | Modify | Add new modules, pass OpenAI API key to state |

#### `src/openai.rs` — OpenAI Client

```
Responsibilities:
- Connect to wss://api.openai.com/v1/realtime with API key
- Handle Realtime API events:
  - session.created → configure session (model, instructions, voice, etc.)
  - session.updated → confirm configuration
  - input_audio_buffer.speech_started → user started speaking
  - input_audio_buffer.speech_stopped → user stopped speaking
  - conversation.item.created → user/assistant message transcript
  - response.audio.delta → streaming audio response
  - response.audio.done → audio chunk complete
  - response.text.delta → streaming text response
  - response.text.done → full text response
  - error → handle errors
- Provide a channel-based interface:
  - outgoing: forward frontend WS messages → OpenAI
  - incoming: forward OpenAI events → frontend WS handler
```

Key types:
```rust
pub enum OpenAiMessage {
    AudioDelta { delta: Vec<u8>, item_id: String },
    TextDone { text: String, item_id: String },
    TranscriptDone { transcript: String, item_id: String },
    Error { message: String },
}

pub struct OpenAiClient {
    pub outgoing_tx: mpsc::Sender<String>,
    pub incoming_rx: mpsc::Receiver<OpenAiMessage>,
}
```

#### `src/conversation.rs` — Conversation Manager

```
Responsibilities:
- Accept incoming WebSocket connection from frontend
- On connect, spawn OpenAI client connection
- Read messages from frontend WS:
  - "audio" messages → forward to OpenAI + buffer for Vosk
  - "commit" → flush Vosk analysis
- Read messages from OpenAI:
  - Forward to frontend WS
- When Vosk results ready:
  - Send pronunciation message to frontend WS
- Handle disconnection cleanup
```

#### WebSocket Protocol (frontend ↔ backend)

```jsonc
// Frontend → Backend
{ "type": "audio_chunk", "data": "<base64 pcm>" }
{ "type": "audio_commit", "data": {} }
{ "type": "session_config", "data": { "instructions": "...", "voice": "alloy" } }

// Backend → Frontend
{ "type": "llm_audio_delta", "data": { "delta": "<base64 pcm>" } }
{ "type": "llm_text_delta", "data": { "delta": "..." } }
{ "type": "llm_text_done", "data": { "text": "..." } }
{ "type": "transcript", "data": { "text": "..." } }
{ "type": "pronunciation", "data": { "words": [...], "overall_score": 85.5 } }
{ "type": "connection_status", "data": { "status": "connected|disconnected|error" } }
```

### Phase 2 — Backend: Pronunciation with Vosk

**Goal:** Route user audio through Vosk in parallel with OpenAI processing.

#### How it works:

1. As audio chunks arrive from frontend, append them to an in-memory `Vec<i16>` buffer
2. On `audio_commit`:
   - Convert buffered samples to WAV in memory with proper header
   - Run through existing `SpeechEngine::create_recognizer()` + `accept_waveform()`
   - Parse `CompleteResultSingle` into `Vec<Word>`
3. Send results to frontend as `{ type: "pronunciation", ... }`

The Vosk pipeline already exists in `src/speech.rs` and `src/api.rs` — we reuse `SpeechEngine` and the `SpeechAnalyzeResult` conversion.

#### Key considerations:
- Vosk operates on 16-bit PCM mono at the original sample rate
- We need a minimal WAV writer (or use the `wav` crate) to construct the in-memory WAV
- Audio should be deduplicated: the same buffer forwarded to OpenAI is used for Vosk

### Phase 3 — Frontend: Conversation View

**Goal:** Build a chat-like conversation interface with pronunciation overlays.

#### Files to modify/create:

| File | Action | Description |
|------|--------|-------------|
| `frontend/components/App.vue` | Modify | Add mode toggle ("Conversation" / "Practice"), conditionally render views |
| `frontend/components/ConversationView.vue` | **Create** | Conversation chat UI with WebSocket client |
| `frontend/components/PracticeView.vue` | **Create** | Extract existing practice UI into its own component |
| `frontend/style.css` | Modify | Add conversation-specific styles |
| `frontend/index.js` | Modify | No changes needed if using component-based approach |
| `frontend/package.json` | Modify | No new deps needed (Vue 3 + Parcel) |

#### `ConversationView.vue` — Component Structure

```
┌─────────────────────────────────────┐
│  Conversation with AI               │
├─────────────────────────────────────┤
│  ┌───────────────────────────────┐  │
│  │ [User Message Bubble]         │  │
│  │  word₁ word₂ word₃ word₄      │  │
│  │  (colored by pronunciation)   │  │
│  └───────────────────────────────┘  │
│                                     │
│  ┌───────────────────────────────┐  │
│  │ [AI Message Bubble]           │  │
│  │  "That's a great attempt!     │  │
│  │   The word 'pronunciation'..."│  │
│  │  [▶ Play Audio]               │  │
│  └───────────────────────────────┘  │
│                                     │
│  ┌───────────────────────────────┐  │
│  │ [User Message]                │  │
│  │  words... (colored)           │  │
│  └───────────────────────────────┘  │
├─────────────────────────────────────┤
│  [🔴 Recording...] / [🎤 Speak]     │
│  Status: Connected ✓                │
└─────────────────────────────────────┘
```

#### State shape:
```js
const conversationMessages = ref([
  {
    role: "user" | "assistant",
    text: "hello world",
    words: [{ text, score, start, end }], // only for user messages
    audioBlob: null,                      // only for assistant messages
    overallScore: 85.5,                   // only for user messages
  }
]);
```

#### Recording flow (Conversation mode):
1. User presses/holds mic button → start `getUserMedia`
2. Create `AudioContext` + `MediaStreamSource`
3. Use `ScriptProcessorNode` or `AudioWorklet` to get raw PCM chunks
4. Base64-encode and send as `audio_chunk` via WebSocket
5. On release → send `audio_commit`
6. Receive `pronunciation` event → update last user message with word data
7. Receive `llm_response` events → append assistant message

For simplicity, we can use the existing `WebAudioRecorder` (WAV) approach, recording short segments, or implement raw PCM chunking. The WAV approach is simpler and we already have it working.

Actually, for the Realtime API, audio is expected as raw PCM 16-bit mono at 24kHz. We need to either:
a) Record raw PCM at 24kHz and stream directly
b) Record at 48kHz and downsample

The OpenAI Realtime API input format: 16-bit PCM mono, 24000 Hz, chunked in base64-encoded frames.

Let's use a `ScriptProcessorNode` or `AudioWorklet` to capture raw PCM and downsample to 24kHz.

#### WebSocket Client (in ConversationView.vue):
```js
let ws = null;
let audioContext = null;
let scriptProcessor = null;
let mediaStream = null;

function connectWebSocket() {
  ws = new WebSocket(`ws://${location.host}/ws/conversation`);
  ws.onmessage = handleMessage;
  ws.onclose = handleDisconnect;
}

function startRecording() {
  // getUserMedia → AudioContext → ScriptProcessorNode
  // onaudioprocess → downsample to 24kHz → base64 → ws.send
}

function handleMessage(event) {
  const msg = JSON.parse(event.data);
  switch (msg.type) {
    case "llm_audio_delta":
      // accumulate audio chunks
      break;
    case "llm_text_done":
      // append assistant message
      break;
    case "pronunciation":
      // update last user message with word data
      break;
    case "transcript":
      // show what OpenAI transcribed
      break;
  }
}
```

### Phase 4 — Polish & Integration

#### System Prompt

Configure the OpenAI Realtime API session with a system prompt that makes it act as a pronunciation-aware conversation partner:

```jsonc
{
  "type": "session.update",
  "session": {
    "instructions": "You are an English conversation partner. Your role is to have natural conversations with the user to help them practice English speaking. The user's pronunciation is being analyzed in real-time. Occasionally (every 2-3 exchanges), give gentle, specific pronunciation tips based on what you observe. Keep your tone friendly and encouraging. Respond naturally — this is a conversation, not a lesson.",
    "voice": "alloy",
    "modalities": ["text", "audio"],
    "input_audio_format": "pcm16",
    "output_audio_format": "pcm16",
    "input_audio_transcription": { "enabled": true }
  }
}
```

#### Mode Toggle

Add a pill/tab toggle in the app header:

```
┌──────────────────────────────────┐
│   ◉ Conversation   ○ Practice    │
└──────────────────────────────────┘
```

- **Conversation**: The new chat UI (default)
- **Practice**: The existing record → analyze → pronunciation feedback UI

#### Existing App.vue Refactor

Split into `PracticeView.vue` (extracted current logic) and `ConversationView.vue` (new). `App.vue` becomes a shell with the mode toggle.

---

## Detailed File Changes

### Backend Changes

#### `Cargo.toml`
```toml
[dependencies]
axum = { version = "0.6.20", features = ["multipart", "ws"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
base64 = "0.21"
tokio-stream = "0.1"
# keep existing deps
```

#### `src/openai.rs` (new — ~200 lines)
- `OpenAiClient::connect(api_key) -> Result<Self>`
- Message loop: read from OpenAI WS → parse events → send via channel
- Write loop: read from channel → serialize → send to OpenAI WS
- Key events handled: `session.created`, `response.audio.delta`, `response.audio.done`, `response.text.delta`, `response.text.done`, `conversation.item.created`, `input_audio_buffer.speech_started/stopped`, `error`

#### `src/conversation.rs` (new — ~250 lines)
- `handle_ws_upgrade(ws: WebSocketUpgrade, state: ServerState) -> impl IntoResponse`
- `handle_socket(stream: WebSocket)`: main coroutine
  - Connect OpenAI client
  - Spawn task for OpenAI → frontend forwarding (with Vosk interleaving)
  - Loop reading frontend messages, forwarding to OpenAI
  - On `audio_chunk`: buffer samples, forward to OpenAI
  - On `audio_commit`: flush Vosk analysis, send result

#### `src/api.rs` (modified — add route)
```rust
.route("/ws/conversation", get(conversation::handle_ws_upgrade))
```

#### `src/main.rs` (modified)
```rust
mod conversation;
mod openai;
// Read OPENAI_API_KEY, pass to ServerState or handle in conversation
```

### Frontend Changes

#### `frontend/components/App.vue` (refactored)
- Add reactive `mode` ref: `"conversation"` | `"practice"`
- Toggle buttons in template
- Conditional render: `<ConversationView />` or `<PracticeView />`
- Extract existing logic into `PracticeView.vue`

#### `frontend/components/PracticeView.vue` (new)
- Exact copy of current `App.vue` logic (recording, analysis, word display, lookup)

#### `frontend/components/ConversationView.vue` (new — ~300 lines)
- WebSocket connection management
- Audio capture (getUserMedia → PCM → WebSocket) using `AudioContext` + `ScriptProcessorNode`
- Audio playback for LLM audio (base64 decode → AudioBuffer → play)
- Message list with word-colored user messages + assistant messages
- Word click → play audio segment (same as existing `playWord`)
- Recording button (push-to-talk style)
- Connection status indicator
- Auto-scroll to bottom on new messages

#### `frontend/style.css` (modified)
- Add `.conversation-view`, `.message-bubble`, `.message-user`, `.message-assistant` styles
- Add `.status-indicator` styles (connected/disconnected/recording)
- Keep existing `.word`, `.good`, `.average`, `.poor`, `.bad` styles (reused)

---

## Key Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Audio streaming format | PCM 16-bit mono 24kHz (OpenAI format) | Realtime API requires this; we downsample from 48kHz mic capture |
| Vosk audio source | Same buffer forwarded to OpenAI | Ensures pronunciation analysis matches what the LLM hears |
| WS protocol | JSON text messages + base64 audio payloads | Simple to debug, extensible |
| Frontend audio capture | `ScriptProcessorNode` (fallback) or `AudioWorklet` | `ScriptProcessorNode` has wider browser support despite being deprecated |
| Audio playback | `AudioContext.decodeAudioData` + `AudioBufferSourceNode` | Gives us control over playback timing and can be stopped |
| Vosk sample rate conversion | Backend side | Frontend sends 24kHz to OpenAI; backend upsamples to match Vosk model's expected rate (typically 16kHz) |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| OpenAI Realtime API latency | Poor UX | Use streaming events (audio.delta) to start playback before response is complete |
| Vosk model size | Slow startup | Already loaded at server start (existing behavior) |
| WebSocket reconnection | Dropped conversation | Implement reconnect with exponential backoff; preserve message history in frontend |
| Vosk + OpenAI transcript mismatch | Confusing UI | Display both; prefer Vosk timing for word coloring, OpenAI transcript for text |
| API key exposure | Security risk | Key stays server-side in the WS proxy |
| Audio format mismatch | No recognition | Add format validation and conversion at the backend boundary |

---

## Future Considerations (Out of Scope)

- **Multi-language support**: Vosk models exist for many languages; OpenAI Realtime API supports multiple languages
- **Conversation history persistence**: Save conversations for later review
- **Pronunciation trend tracking**: Track improvement over time across conversations
- **Custom voices**: OpenAI Realtime API supports `alloy`, `echo`, `fable`, `onyx`, `nova`, `shimmer`
- **Function calling**: Could add a `check_pronunciation` function that the LLM calls explicitly when it wants to give feedback
