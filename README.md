# conlang

Construct languages from phonetics, to orthography, to grammar, to lexicon, to completion.

The Conlang kit assists in the creation of constructed languages.
On a simple level, it can be used as a name, word, or sentence generator: you describe a phoneme
inventory and phonotactic patterns in a JSON sketch, and it produces words and sentences in IPA.
Beyond that, sketches can describe word classes, morphology (affixation), sentence-template
grammars, and lexicons.

What distinguishes this toolkit from others is the focus on phonetics.
**It is meant to get you from zero to hearing a language spoken as quickly as possible.**
Generated output is IPA, and a text-to-speech voice driver can speak it aloud, which allows rapid
iteration on sounds and patterns until the language sounds like what you want.

## Quick start

```sh
cargo run --features voice-polly -- config voice init
cargo run --features voice-polly -- generate-sentences --config examples/english.json --speak
```

This generates sentences from the English-like sketch in
[`examples/english.json`](examples/english.json), prints them in IPA, and speaks each one aloud
through [AWS Polly](doc/voices/polly.md).

To hear audio, you need:

- A voice driver compiled in: `--features voice-polly` (cloud neural TTS, requires AWS credentials)
  or `--features voice-espeak` (free, offline, requires the system eSpeak-ng library).
- A voice configuration: scaffold one with `conlang config voice init`

Without a voice feature or the `--speak` flag, generation still works and prints IPA; there is just
no audio. Simpler sketches to start from live in [`examples/`](examples/).

## The CLI

- `generate-words`: generate words from a sketch (or inline `--consonants`/`--vowels`/`--pattern`
  flags), optionally restricted to one word class.
- `generate-sentences`: generate sentences using the sketch's grammar templates, or unstructured
  strings of words.
- `config voice init | list | scan | test`: scaffold, inspect, discover, and try out text-to-speech
  voices.

Pass `--speak` to the generators to hear the output with a random configured voice, or
`--speak-with <name>` for a specific one.

## Documentation

The [full documentation](https://tgockel.github.io/conlang/) gives a deep dive into the fundamentals
of spoken language and how this maps to configuration files.

## License

Apache-2.0
