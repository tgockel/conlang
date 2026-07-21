# Vowel Reduction

Vowel reduction collapses the vowels of unstressed syllables to a weaker, more central vowel (e.g.:
schwa `ə`).
This is characteristic of stress-timed languages: English pronounces `banana` as /bəˈnænə/ rather
than /baˈnana/, Russian merges unstressed /o/ and /a/ (*молоко* /məlɐˈko/), European Portuguese
reduces unstressed vowels so far they often nearly disappear, and Central Catalan collapses
unstressed /a/, /e/, and /ɛ/ to [ə].
Syllable-timed languages like Spanish, standard Italian, and Japanese instead keep full vowel
quality everywhere; leave reduction off for languages with that rhythm.

Natural languages vary in what unstressed vowels reduce *to* (Russian [ɐ], European Portuguese [ɨ],
Catalan [u] for back vowels) and in which vowels reduce at all.
The `target` and `targets` fields configure both aspects.

## Configuration

Add a `reduction` section to your config JSON:

```json
{
  "reduction": { "probability": 0.7, "target": "ə" }
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `probability` | number | Yes | Probability that an unstressed syllable's vowel reduces, between 0.0 and 1.0. |
| `target` | string | One of `target`/`targets` | Single vowel that every reduced vowel collapses to. |
| `targets` | object | One of `target`/`targets` | Map from source vowels to the vowel they reduce to. Only mapped vowels reduce. |

Exactly one of `target` and `targets` must be present.
If the `reduction` section is absent, no reduction occurs.

### Targets

`target` is the simple form: every vowel in a reduced syllable collapses to that one vowel, as in
the English-style example above.

`targets` gives per-vowel control.
Each key is a string of one or more IPA vowels; the value is the single vowel they reduce to.
Only the listed vowels reduce — everything else keeps its quality even in unstressed syllables.
A vowel may appear in at most one key.

A Central Catalan–style system, where /a e ɛ/ merge to [ə] and /o ɔ/ raise to [u] while the high
vowels /i u/ never reduce:

```json
{
  "reduction": {
    "probability": 1.0,
    "targets": { "aeɛ": "ə", "oɔ": "u" }
  }
}
```

A Russian-style *akanye*, merging unstressed /o/ and /a/ into [ɐ]:

```json
{
  "reduction": {
    "probability": 0.9,
    "targets": { "oa": "ɐ" }
  }
}
```

### Interaction with Stress

Reduction targets whatever syllables are left unstressed by [stress assignment](./stress.md), so
which syllables are eligible is determined by the `stress` configuration:

- Syllables carrying primary stress never reduce.
  This includes monosyllabic words, which always receive primary stress (unless the strategy is
  `none`).
- A class with `"stress": "none"` has every syllable eligible, so the word can reduce entirely.
- With `"secondary": true`, alternating syllables carry secondary stress and are also protected,
  halving the eligible syllables in longer words.

Because of this dependency, a sketch with a `reduction` section but no stress configuration at all
(no top-level `stress` and no per-class `"stress"` fields) is rejected as a validation error.
Without stress, every vowel is a candidate for reduction, which leads to an undirected mush of
whatever your target is.
If you still want this, you can explicitly write `"stress": "none"` at the top-level or per-class.

## Behavior

- Reduction runs after [stress assignment](./stress.md), so it depends on the final stress marking
  of each word.
- Only fully unstressed syllables are eligible.
  Syllables with primary or secondary stress never reduce.
  Secondary stress protects vowel quality, as it does in English (compare the final syllables of
  `ˈal.pha.ˌbet` and `ˈtrum.pet`).
- Word classes with `"stress": "none"` have every syllable unstressed, so function-word classes
  (determiners, pronouns, prepositions) reduce throughout — giving the "the → ðə, of → əv" effect.
- The whole vocalic nucleus reduces: diacritics on the vowel are dropped, and a diphthong collapses
  to a single vowel.
  With `targets`, a diphthong is keyed by its first component (so with `"aeɛ": "ə"`, /ai/ collapses
  to [ə] but /ui/ is untouched).
  Consonants in the onset and coda are untouched.
- Reduction applies to a word's lexicon entry when the vocabulary is pre-generated, so a reduced
  word form is stable across a session.
- A reduction target may appear in output even if it is not in the configured vowel inventory;
  targets are outcomes of reduction, not inventory phonemes.
