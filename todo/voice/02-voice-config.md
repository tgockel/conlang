# Feature: Voice Configuration File

## Problem

Voice preferences are orthogonal to language design. Which TTS driver to use, which
voice name, what speaking rate -- these depend on what's installed on the user's machine
and what sounds good to them, not on the conlang's phonology. They should not live in the
sketch JSON alongside consonants and grammar rules.

Currently, `create_default_driver()` in `src/voice/mod.rs` hardcodes the voice selection
(Polly Joanna Neural, or eSpeak mb-de5). There is no way to configure voice parameters
without editing source code.

## Design

A standalone JSON configuration file defines named voices. Each voice entry specifies a
driver and driver-specific parameters. Generation commands reference voices by name via
`--speak-with <name>` (see 03-cli-integration).

### Config Location

Search order:
1. `--voice-config <path>` CLI flag (explicit override)
2. `./voices.json` (project-local)
3. `~/.config/conlang/voices.json` (user-level default)

If no config file is found and `--speak-with` is used, emit a helpful error pointing
the user to `conlang config voice` (see 03-cli-integration).

### Config Format

```json
{
  "voices": {
    "polly-joanna": {
      "driver": "polly",
      "voice_id": "Joanna",
      "engine": "neural"
    },
    "polly-matthew": {
      "driver": "polly",
      "voice_id": "Matthew",
      "engine": "neural"
    },
    "espeak-de5": {
      "driver": "espeak",
      "voice": "mb-de5",
      "rate": 120,
      "pitch": 50,
      "volume": 100
    },
    "espeak-default": {
      "driver": "espeak",
      "voice": "en",
      "rate": 150
    }
  },
  "default": "espeak-de5"
}
```

- `voices` is a map from name to voice config.
- `default` is an optional default voice name. Used when `--speak-with` is given without
  a value, or when `--speak` is used as a shorthand (backward compat).
- Each entry has `"driver"` as a discriminator, with remaining fields driver-specific.
- The `"engine"` field under Polly entries refers to Polly's own engine concept
  (neural vs standard), not our driver abstraction.
- Unknown fields are ignored (forward-compatible for new drivers or driver features).

### Driver-Specific Parameters

**Polly** (`"driver": "polly"`):
| Field      | Type   | Default    | Description                          |
|------------|--------|------------|--------------------------------------|
| `voice_id` | string | `"Joanna"` | AWS Polly voice identifier           |
| `engine`   | string | `"neural"` | Polly engine: `"neural"` or `"standard"` |

**eSpeak-ng** (`"driver": "espeak"`):
| Field    | Type   | Default       | Description                             |
|----------|--------|---------------|-----------------------------------------|
| `voice`  | string | `"en"`        | eSpeak voice name (e.g., `"mb-de5"`)   |
| `rate`   | int    | `175`         | Words per minute (80-450)               |
| `pitch`  | int    | `50`          | Pitch (0-99)                            |
| `volume` | int    | `100`         | Volume (0-200, >100 may clip)           |

## Implementation

### `src/voice/config.rs`

```rust
#[derive(Deserialize)]
pub struct VoiceConfigFile {
    voices: HashMap<String, VoiceEntry>,
    default: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "driver")]
enum VoiceEntry {
    #[serde(rename = "polly")]
    Polly(PollyConfig),
    #[serde(rename = "espeak")]
    Espeak(EspeakConfig),
}

#[derive(Deserialize)]
struct PollyConfig {
    voice_id: Option<String>,
    engine: Option<String>,
}

#[derive(Deserialize)]
struct EspeakConfig {
    voice: Option<String>,
    rate: Option<i32>,
    pitch: Option<i32>,
    volume: Option<i32>,
}
```

### Loading

A `load_voice_config(cli_path: Option<&Path>) -> Result<VoiceConfigFile>` function
implements the search order. Each driver's `Voice` implementation gains a `from_config()`
constructor that accepts the driver-specific config struct.

### Integration with Existing Types

- `PollyVoice::from_config(config: &PollyConfig)` -- maps `voice_id` string to
  `aws_sdk_polly::types::VoiceId` and `engine` string to `aws_sdk_polly::types::Engine`
- `EspeakVoice::from_config(config: &EspeakConfig)` -- calls `EspeakVoice::new()` with
  the voice name, then applies rate/pitch/volume via `espeak_SetParameter`

## Verification

- A `voices.json` with a Polly entry and an eSpeak entry loads correctly
- Missing optional fields fall back to defaults
- Unknown driver type produces a clear error
- Unknown fields in an entry are silently ignored
- Config search order works (CLI > project-local > user-level)
- Missing config with `--speak-with` prints helpful guidance

## Notes

Depends on 01-voice-module (implemented) for the driver implementations. This feature is
purely about config loading and parsing -- the CLI integration is in 03-cli-integration.
