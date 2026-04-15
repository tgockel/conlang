# Feature: Voice CLI Integration

## Problem

The current `--speak` flag is a boolean that uses `create_default_driver()` to pick a
hardcoded driver. Users cannot select a voice without recompiling. There is also no way
to discover what TTS drivers are compiled in or what voices are available on their system
(e.g., which eSpeak-ng MBROLA voice databases are installed).

## Design

Two changes to the CLI:

1. **Generation commands** get `--speak-with <name>` to select a configured voice.
2. **A new `conlang config voice` subcommand** for discovery and management.

### `--speak-with` on Generation Commands

```
conlang generate-words --config lang.json --speak-with espeak-de5
conlang generate-sentences --config lang.json --speak-with polly-joanna
```

- `<name>` refers to an entry in the voice config file (see 02-voice-config).
- If `--speak-with` is given without a value, uses the `default` from the config file.
- `--speak` (no argument) picks a random voice from the configured voices each time it
  speaks. This mirrors the randomized generation philosophy elsewhere in the tool and
  lets users hear variety without committing to one voice.

### `conlang config voice` Subcommand

```
conlang config voice list
conlang config voice scan
conlang config voice test <name> [ipa]
```

#### `list`

Prints the configured voices from the voice config file:

```
Configured voices (from ~/.config/conlang/voices.json):
  polly-joanna    polly    Joanna (neural)
  espeak-de5      espeak   mb-de5 (rate: 120)
  espeak-default  espeak   en (rate: 150)

Default: espeak-de5
```

If no config file exists, prints guidance on creating one.

#### `scan`

Discovers available drivers and voices on the system. For each compiled-in driver:

**Polly**: Reports whether AWS credentials are configured (does not list all possible
voices -- that list is static and enormous).

```
polly:
  AWS credentials: configured (region: us-east-1)
  Voices: see https://docs.aws.amazon.com/polly/latest/dg/voicelist.html
```

**eSpeak-ng**: Enumerates installed voices by calling `espeak_ListVoices()`, with special
attention to MBROLA voices (checking whether the corresponding MBROLA data files exist):

```
espeak:
  Installed voices:
    en          English
    de          German
  MBROLA voices (data installed):
    mb-de1      mb-de2      mb-de3      mb-de4
    mb-de5      mb-de6      mb-de7      mb-de8
  MBROLA voices (definition exists, data missing):
    mb-en1      mb-fr1      mb-fr2      ...
```

This helps users know what they can put in their config file without guessing.

#### `test`

Speaks a test phrase with a specific voice:

```
conlang config voice test espeak-de5
conlang config voice test espeak-de5 "ˈpa.ta"
```

- If IPA is provided, speaks that.
- If omitted, uses a built-in test phrase covering a range of common phonemes
  (e.g., `"ˈpa.ta ˈka.ba ˈda.ɡa ˈsa.ʃa ˈma.na"`).
- Uses `voice::AudioSink` and the named voice's `Voice::speak()` implementation.
- Useful for quickly auditioning voices without running a full generation.

## Implementation

### `src/main.rs`

The top-level `Command` enum gains a `Config` variant:

```rust
enum Command {
    GenerateWords(GenerateWordsCmd),
    GenerateSentences(GenerateSentencesCmd),
    Config(ConfigCmd),
}

#[derive(Parser)]
enum ConfigCmd {
    Voice(VoiceCmd),
}

#[derive(Parser)]
enum VoiceCmd {
    List,
    Scan,
    Test(VoiceTestCmd),
}

#[derive(Parser)]
struct VoiceTestCmd {
    name: String,
    ipa: Option<String>,
}
```

### `GenerateWordsCmd` / `GenerateSentencesCmd`

Replace the `speak: bool` field:

```rust
/// Speak with a configured voice. Use the default voice if no name given.
#[arg(long)]
pub speak_with: Option<Option<String>>,
```

This gives three states:
- Flag absent: no speech
- `--speak-with`: use default voice
- `--speak-with espeak-de5`: use named voice

`--speak` selects a random voice from all configured voices each time it speaks.
This is not deprecated -- it's a first-class mode that mirrors the randomized
generation philosophy elsewhere in the tool.

### Speaker Construction

Replace the current `create_default_driver()` call with config-driven construction:

```rust
let config = voice::config::load_voice_config(cmd.voice_config.as_deref())?;
let sink = voice::AudioSink::new()?;
let driver: Box<dyn voice::Voice> = config.build_voice(&voice_name).await?;
```

### `conlang config voice scan` (eSpeak-ng)

Uses `espeak_ListVoices()` from `espeakng-sys` to enumerate installed voices.
Cross-references MBROLA voice definitions in the eSpeak data directory against
installed MBROLA data directories to distinguish "available" from "needs data install".

### `conlang config voice scan` (Polly)

Checks `aws_config::load_defaults()` to see if credentials resolve. Does not
enumerate all Polly voices (there are hundreds and they're region-dependent).

## Verification

- `conlang config voice list` shows configured voices from the config file
- `conlang config voice scan` reports available drivers and voices
- `conlang config voice test espeak-de5` plays audio
- `conlang config voice test espeak-de5 "ˈpa.ta"` plays the given IPA
- `conlang generate-words --config lang.json --speak-with espeak-de5` speaks each word
- `conlang generate-words --config lang.json --speak-with` uses the default voice
- `conlang generate-words --config lang.json --speak` picks a random voice each time
- Building with only `voice-espeak` feature: Polly voices in config produce a clear
  "driver not compiled" error

## Notes

Depends on 01-voice-module (implemented) and 02-voice-config. This is the user-facing
surface that ties the other pieces together.
