# Stress

Generated words can receive IPA stress marks (ˈ for primary, ˌ for secondary) according to a configurable strategy.
Stress is assigned after syllable generation, so it works with both fresh generation and lexicon-based sampling.
When [morphology](morphology.md) is configured, affixation runs first, so stress is computed over the full derived form.
Syllables left unstressed are eligible for [vowel reduction](reduction.md).

## Top-Level Configuration

Add a `"stress"` field to your sketch to set the default strategy for all word classes:

```json
{
  "stress": {
    "default": "trochaic",
    "secondary": true
  }
}
```

`"default"` is the stress strategy applied to every word class that does not override it.
`"secondary"` enables secondary stress on alternating syllables in words with 3 or more syllables.
It defaults to `false` when omitted.

## Strategies

| Strategy      | Primary stress position          | Typical languages |
|:--------------|:---------------------------------|:------------------|
| `trochaic`    | First syllable                   | English, German   |
| `iambic`      | Second syllable (first if mono)  | French-ish        |
| `penultimate` | Second-to-last syllable          | Latin, Spanish    |
| `final`       | Last syllable                    | French, Turkish   |
| `none`        | No stress assigned               | Function words    |

Monosyllabic words receive primary stress unless the strategy is `none`.

## Per-Class Overrides

Each [word class](grammar.md#word-classes) can override the default strategy with its own `"stress"` field.
This is how you keep function words unstressed while content words carry stress:

```json
{
  "stress": { "default": "trochaic", "secondary": true },
  "word_classes": {
    "det":  { "patterns": ["CV"], "stress": "none" },
    "noun": { "patterns": ["$OV$K", "$OV$K $OV$K"] },
    "verb": { "patterns": ["$OV$K"], "stress": "penultimate" }
  }
}
```

Here, determiners get no stress, nouns inherit the top-level `trochaic` strategy, and verbs use `penultimate`.

## Secondary Stress

When `"secondary": true` and a word has 3 or more syllables, secondary stress (ˌ) is placed on alternating
syllables aligned with the primary stress, so the word parses into uniform two-syllable feet and
secondary stress never lands adjacent to the primary.

- `trochaic` builds strong-weak feet from the start of the word: ˈta.ka.ˌta.ka
- `iambic` builds weak-strong feet: ta.ˈta.ta.ˌta
- `penultimate` and `final` alternate outward from the primary: ta.ˌta.ta.ˈta.ta and ta.ˌta.ta.ˈta

## Without Stress Configuration

When no `"stress"` field is present at either level, no stress is assigned -- words are generated without stress marks.

Note that omitting `"stress"` is distinct from setting `"stress": "none"`.
Both produce unstressed words, but `"none"` is meaningful as a per-class override: it keeps specific word classes
unstressed when a top-level default would otherwise give them stress.
This is common for function words; for example, in English, articles like "a" `/ə/` and "the" `/ðə/` and prepositions
like "to" `/tu/` and "of" `/ʌv/` carry no stress.
