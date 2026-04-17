# eSpeak-ng

[eSpeak-ng](https://github.com/espeak-ng/espeak-ng) is an open-source speech synthesizer that runs entirely on your
machine.
It produces robotic, formant-based speech -- not the most natural-sounding output, but it is free, fast, and works
offline, making it ideal for rapid iteration on a language's sound.

> **NOTE**
>
> If you just want the configuration fields, skip to [Configuration](#configuration).

## Setup

Enable the feature flag when building:

```sh
cargo build --features voice-espeak
```

The `espeak-ng` system library must be installed on your machine.
On Debian and Ubuntu, the package is `espeak-ng-data`; on other distributions, look for `espeak-ng` in your
package manager.

You can verify the installation by running `conlang config voice scan`, which lists all voices that eSpeak-ng
can find.

## Configuration

Each eSpeak voice entry requires `"driver": "espeak"` and accepts the following optional fields:

| Field    | Type    | Default      | Description                                              |
|:---------|:--------|:-------------|:---------------------------------------------------------|
| `driver` | string  | *(required)* | Must be `"espeak"`                                       |
| `voice`  | string  | `"en"`       | eSpeak voice or language code (e.g., `"en"`, `"mb-de5"`) |
| `rate`   | integer | *(none)*     | Speech rate                                              |
| `pitch`  | integer | *(none)*     | Pitch adjustment (0-99)                                  |
| `volume` | integer | *(none)*     | Volume level (0-200)                                     |

A minimal entry:

```json
{
  "driver": "espeak"
}
```

This uses the English voice at the default rate, pitch, and volume.

A more complete entry:

```json
{
  "driver": "espeak",
  "voice": "mb-de5",
  "rate": 120,
  "pitch": 50,
  "volume": 100
}
```

## MBROLA Voices

Voice codes starting with `mb-` (like `mb-de5` or `mb-en1`) use the [**MBROLA**](https://github.com/numediart/MBROLA)
speech synthesis system through eSpeak-ng.
MBROLA voices produce smoother, more natural output than eSpeak-ng's default formant synthesis, but they require
separate data files to be installed on the system.

Run `conlang config voice scan` to see which MBROLA voices are available.
The scan distinguishes between:

* **Data installed** -- the voice is ready to use
* **Definition exists, data missing** -- eSpeak-ng knows about the voice, but the data package is not installed

Install the data package for the voice you want (e.g., `mbrola-de5` on Debian/Ubuntu) and it will appear in the
"data installed" list on the next scan.

## IPA Conversion

The toolkit generates IPA strings, but eSpeak-ng uses its own phoneme notation internally.
When you speak through an eSpeak voice, the toolkit automatically converts IPA to eSpeak's Kirshenbaum-based
phoneme format.

This mapping covers the full consonant and vowel inventory described in [Phonemes](../lang/phonemes.md),
including compound segments like affricates (/t͡ʃ/, /d͡ʒ/) and common diacritics (aspiration, nasalization).
Some less common diacritics and tone markers have partial or approximate support -- if a particular sound does not
come through as expected, try a nearby phoneme or switch to [AWS Polly](polly.md), which accepts IPA directly.
