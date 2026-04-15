# Feature: Voice Module and Driver Abstraction

**Status: Implemented**

## Problem

`src/speak.rs` was a monolithic file that hardcoded both AWS Polly and eSpeak-ng behind a
single `pronounce` cargo feature. This had several problems:

- Adding a new driver required every user to have every driver's system dependencies
  installed (e.g., `libespeak-ng-dev`, AWS credentials) even if they only wanted one.
- Voice selection was hardcoded at compile time. The trial code looped through
  all voices rather than letting the user pick one.
- The Polly and eSpeak-ng code were interleaved with shared audio playback logic,
  making it hard to add or remove a driver cleanly.

## Design

A `Voice` trait abstracts over TTS drivers. Each driver is its own module behind its own
cargo feature. Shared concerns (audio device, PCM playback) live in common code that any
driver can use.

**Terminology note:** We use "driver" for our TTS backend abstraction to avoid confusion
with AWS Polly's own "engine" concept (neural vs standard).

### Trait

```rust
pub trait Voice {
    /// Speak an IPA string through the given audio sink.
    fn speak(&self, ipa: &str, sink: &AudioSink) -> Result<(), anyhow::Error>;
}
```

The trait is **synchronous**. The Polly driver handles its async AWS calls internally
via `tokio::task::block_in_place` + `tokio::runtime::Handle::current().block_on()`.

The trait takes IPA as input. Each driver is responsible for converting IPA into
whatever its backend expects (SSML for Polly, Kirshenbaum `[[...]]` for eSpeak-ng).

### Audio Sink

A shared `AudioSink` struct owns the `rodio::Player` and `rodio::MixerDeviceSink`.
It provides:

- `play_ogg(&self, data: &[u8])` -- decode and play OGG/Vorbis (used by Polly)
- `play_pcm(&self, samples: &[i16], sample_rate: u32)` -- play raw PCM (used by eSpeak-ng)

Drivers receive the sink by reference so callers can reuse a single sink across
multiple drivers.

### Driver Implementations

**`PollyVoice`** (feature: `voice-polly`)
- Wraps `aws_sdk_polly::Client`
- Configurable: `voice_id` (Joanna, Matthew, etc.), `engine` (Neural, Standard)
- Converts IPA to SSML `<phoneme alphabet="ipa" ph="...">` tags

**`EspeakVoice`** (feature: `voice-espeak`)
- Wraps the `espeakng-sys` FFI
- Configurable: voice name (e.g., `mb-de5`)
- Converts IPA to eSpeak phoneme codes via `ipa_to_espeak_phonemes()`
- Manages the global eSpeak-ng singleton via `Mutex<Option<u32>>` (initialize once,
  set voice per call)

### Factory Function

`voice::create_default_driver()` constructs a default `Box<dyn Voice>`. It prefers
Polly when `voice-polly` is enabled, otherwise falls back to eSpeak-ng with `mb-de5`.
This is a temporary default until 03-cli-integration adds proper voice selection.

### Cargo Features

```toml
[features]
default = []
voice-polly = ["dep:aws-config", "dep:aws-sdk-polly", "dep:rodio"]
voice-espeak = ["dep:espeakng-sys", "dep:rodio"]
```

Users enable only the drivers they want:
- `cargo build --features voice-polly` -- only Polly, no eSpeak system deps
- `cargo build --features voice-espeak` -- only eSpeak, no AWS deps
- `cargo build --features voice-polly,voice-espeak` -- both

The old `pronounce` feature has been removed.

## Implementation

### Module Structure

```
src/voice/
  mod.rs        -- Voice trait, AudioSink, create_default_driver()
  polly.rs      -- PollyVoice (behind #[cfg(feature = "voice-polly")])
  espeak.rs     -- EspeakVoice (behind #[cfg(feature = "voice-espeak")])
```

The voice module is part of the library (`pub mod voice` in `src/lib.rs`), gated
behind `#[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]`.

### `src/voice/mod.rs`

- `Voice` trait definition (sync)
- `AudioSink` struct with `player: rodio::Player` and `_mixer: rodio::MixerDeviceSink`
- `create_default_driver()` async factory function

### `src/voice/polly.rs`

- `PollyVoice` struct with `client`, `voice_id`, `engine` fields
- `PollyVoice::new(voice_id, engine)` -- async, loads AWS config
- `Voice::speak()` -- uses `block_in_place`/`block_on` to call the async Polly API

### `src/voice/espeak.rs`

- `ESPEAK_BUFFER: Mutex<Vec<i16>>` and `ESPEAK_INIT: Mutex<Option<u32>>` statics
- `espeak_callback` -- unsafe extern "C" callback for sample accumulation
- `ensure_init()` -- initializes the global singleton, protected by mutex
- `set_voice()`, `synthesize()` -- module-private helpers
- `ipa_to_espeak_phonemes()` -- the full IPA-to-Kirshenbaum mapping
- `EspeakVoice` struct with `voice` name and `sample_rate`

### `src/main.rs`

- Uses `conlang::voice` (cfg-gated import)
- Speaker is `Option<(voice::AudioSink, Box<dyn voice::Voice>)>`
- Both `GenerateWords` and `GenerateSentences` share the same speaker pattern
- Error message when `--speak` is used without a voice feature compiled in

### Migration

`src/speak.rs` has been deleted. Its functionality is split across
`src/voice/mod.rs`, `src/voice/polly.rs`, and `src/voice/espeak.rs`.

## Verification

- `cargo build --features voice-polly` compiles without eSpeak system deps
- `cargo build --features voice-espeak` compiles without AWS deps
- `cargo build --no-default-features` compiles with no voice support (no rodio, no TTS)
- `cargo build --features voice-polly,voice-espeak` compiles with both
- `--speak` functionality works with the appropriate feature enabled

## Notes

This is the foundation -- 02-voice-config and 03-cli-integration depend on this.
