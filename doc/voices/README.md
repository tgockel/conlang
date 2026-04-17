# Voices

The whole point of this toolkit is to get from zero to hearing a language spoken.
The [language fundamentals](../lang/README.md) pages cover how to define sounds, syllable shapes, and grammar --
voices are the bridge from IPA on screen to audio in your ears.

> **NOTE**
>
> If you already have a voice driver compiled and configured, skip to [CLI Usage](#cli-usage) for the generation
> flags or [Configuration](#configuration) for the JSON schema.
> For driver-specific setup, see [eSpeak-ng](espeak.md) or [AWS Polly](polly.md).

## How It Works

When you generate words or sentences, the toolkit produces **IPA strings** -- sequences of phonetic symbols like
/ˈpa.ta/ or /ˈka.ba.da/.
A **voice driver** takes those IPA strings and converts them into audible speech through text-to-speech synthesis.

Two drivers are available:

* **[eSpeak-ng](espeak.md)** -- open-source, runs locally, no network required
* **[AWS Polly](polly.md)** -- cloud-based neural TTS with natural-sounding output

Drivers are compile-time **feature flags**, so you choose which to include when building.
You then define **named voices** in your configuration file, each pointing at a driver with specific settings.
The CLI references voices by name.

## Getting Started

The fastest path to hearing your language is eSpeak-ng -- it runs locally with no cloud account.

Build with the eSpeak feature flag:

```sh
cargo build --features voice-espeak
```

Create a `conlang.json` in your project directory with a single voice:

```json
{
  "voices": {
    "definitions": {
      "my-voice": {
        "driver": "espeak",
        "voice": "en",
        "rate": 150
      }
    },
    "default": "my-voice"
  }
}
```

Generate some words and hear them:

```sh
conlang generate-words --consonants ptkmnsl --vowels aeiou --pattern "CVC" --speak
```

The `--speak` flag picks from all configured voices (here, just the one).
To select a specific voice by name, use `--speak-with my-voice` instead.

## Configuration

### Config File Location

The toolkit searches for voice configuration in this order:

| Priority | Location                  | Notes                     |
|:---------|:--------------------------|:--------------------------|
| 1        | `--voice-config <path>`   | Explicit CLI override     |
| 2        | `./conlang.json`          | Project-local config      |
| 3        | `~/.conlang/config.json`  | User-level config         |

The `"voices"` section can coexist with language sketch data in the same file -- the tool ignores sections it
does not need for a given command.

### JSON Structure

A complete example with multiple voices across both drivers:

```json
{
  "voices": {
    "definitions": {
      "polly-joanna": {
        "driver": "polly",
        "voice_id": "Joanna",
        "engine": "neural"
      },
      "polly-matthew": {
        "driver": "polly",
        "voice_id": "Matthew",
        "engine": "standard"
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
}
```

`"definitions"` is a map of voice names to driver configurations.
The names are yours to choose -- they are the values you pass to `--speak-with` on the command line.

`"default"` sets which voice to use when `--speak-with` is passed the value `"default"` or when no name is
otherwise specified.
It must match one of the keys in `"definitions"`.

Each entry requires a `"driver"` field (`"espeak"` or `"polly"`) and accepts driver-specific fields described
on the [eSpeak-ng](espeak.md) and [AWS Polly](polly.md) pages.

### Choosing a Driver

|                  | eSpeak-ng                    | AWS Polly                    |
|:-----------------|:-----------------------------|:-----------------------------|
| Cost             | Free                         | Pay-per-character            |
| Network          | Offline                      | Requires internet            |
| Voice quality    | Robotic / formant            | Neural / natural             |
| Latency          | Immediate                    | Network round-trip           |
| Setup            | Install system package       | Configure AWS credentials    |
| IPA fidelity     | Approximate (via conversion) | Direct (via SSML)            |

**eSpeak-ng** is the best choice for rapid iteration -- no network, no cost, instant feedback.
**AWS Polly** shines when you want to hear something closer to natural speech, such as for demos or recordings.

## CLI Usage

### Speaking Generated Output

The `--speak-with` and `--speak` flags work on both `generate-words` and `generate-sentences`:

```sh
conlang generate-words --config sketch.json --speak-with default

conlang generate-words --config sketch.json --speak-with espeak-de5

conlang generate-words --config sketch.json --speak
```

`--speak-with <NAME>` speaks each generated item with the named voice.
The special value `"default"` resolves to whatever `"default"` is set to in the config.

`--speak` speaks each generated item with a randomly chosen voice from all configured voices.
This is useful when you have defined multiple voices and want to hear variety.

The two flags are mutually exclusive.

To point at a config file outside the normal search order, add `--voice-config <PATH>`:

```sh
conlang generate-words --config sketch.json --speak \
    --voice-config ~/my-voices.json
```

### Voice Management Commands

Three subcommands under `conlang config voice` help you inspect and test your setup:

**List** configured voices:

```sh
conlang config voice list
```

Shows each voice name, its driver, and key settings.
Useful for verifying that your config file is being found and parsed.

**Scan** for available drivers and voices on the system:

```sh
conlang config voice scan
```

For eSpeak-ng, this shows installed standard voices and MBROLA voices (distinguishing between those with and
without data files).
For Polly, it queries the AWS API and lists available voice IDs for your configured region.

**Test** a voice with a built-in or custom phrase:

```sh
conlang config voice test espeak-de5

conlang config voice test polly-joanna "ˈka.ta ˈba.da"
```

When no IPA text is given, the default test phrase is `ˈpa.ta ˈka.ba ˈda.ɡa ˈsa.ʃa ˈma.na` -- a sequence that
exercises common plosives, fricatives, and nasals.

## Troubleshooting

**"no voice driver compiled in"** -- Neither `voice-espeak` nor `voice-polly` was enabled at compile time.
Rebuild with `cargo build --features voice-espeak` (or `voice-polly`, or both).

**"this voice uses the Polly driver, but Polly support was not compiled in"** (or the eSpeak equivalent) --
Your config references a driver that was not enabled.
Either change the `"driver"` in the config or rebuild with the needed feature flag.

**"no conlang configuration file found"** -- The tool could not find `./conlang.json` or
`~/.conlang/config.json`.
Create one or use `--voice-config <path>`.

**"unknown voice"** -- The name passed to `--speak-with` does not match any key in `"definitions"`.
Run `conlang config voice list` to see available names.

For driver-specific issues (system libraries, MBROLA data, AWS credentials), see the
[eSpeak-ng](espeak.md) and [AWS Polly](polly.md) pages.
