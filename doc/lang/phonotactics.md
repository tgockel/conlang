# Phonotactics

Phonotactics define which sounds can appear where in a syllable or word.
In this toolkit, you express phonotactic rules as **pattern strings** that describe the structure of generated syllables.

## Pattern Codes

### Basic

| Code | Meaning                          |
|:----:|:---------------------------------|
| `C`  | Any consonant from the inventory |
| `V`  | Any vowel from the inventory     |

Spaces separate syllables within a word.
For example, `CVC CV` generates a two-syllable word: one CVC syllable followed by one CV syllable.

### Place of Articulation

These codes select consonants by where in the vocal tract the sound is produced.
See the [phonemes](phonemes.md) consonant table for the full mapping.

| Code | Places                          |
|:----:|:--------------------------------|
| `M`  | Bilabial                        |
| `L`  | Labiodental                     |
| `D`  | Dental, Alveolar, Post alveolar |
| `Ḍ`  | Retroflex                       |
| `J`  | Palatal                         |
| `G`  | Velar                           |
| `Q`  | Uvular                          |
| `H`  | Pharyngeal, Glottal             |

### Manner of Articulation

These codes select consonants by how the sound is produced.

| Code | Manners                           |
|:----:|:----------------------------------|
| `P`  | Plosive                           |
| `N`  | Nasal                             |
| `T`  | Trill, Tap, Flap                  |
| `X`  | Fricative, Lateral Fricative      |
| `R`  | Approximant, Lateral Approximant  |
| `A`  | Affricate                         |

### Bracket Sets `[...]`

Brackets specify an exact set of IPA phonemes to choose from, regardless of the inventory.

* `[ptk]VC`: one of /p/, /t/, or /k/, then any vowel, then any consonant
* `C[aiu]C`: any consonant, then one of /a/, /i/, or /u/, then any consonant

The contents are parsed as IPA, so compound segments and diacritics work:

* `[t͡ʃd͡ʒ]VC`: one of /t͡ʃ/ or /d͡ʒ/, then vowel, then consonant
* `[tʰpʰkʰ]V`: one of aspirated /t/, /p/, or /k/, then vowel

### Optional Segments `(...)`

Parentheses make a single position optional -- it has a 50% chance of appearing.

* `(C)VC`: optional onset -- generates either VC or CVC
* `CV(C)`: optional coda -- generates either CV or CVC
* `(C)V(C)`: both optional -- generates V, VC, CV, or CVC

Brackets work inside parentheses:

* `([ptk])VC`: optionally one of /p/, /t/, or /k/, then vowel, then consonant

## Examples

A simple CV language:

```sh
conlang generate-words --consonants ptkmnsl --vowels aeiou --pattern "CV"
```

A language with CVC syllables and optional onsets:

```sh
conlang generate-words --consonants ptkbdgmnŋsʃlr --vowels aeiou \
    --pattern "(C)VC"
```

Two-syllable words with specific initial consonants:

```sh
conlang generate-words --consonants ptkbdgmnŋsʃlr --vowels aeiou \
    --pattern "[ptkbdg]VC CV"
```

Nasal-only codas with optional fricative onsets:

```sh
conlang generate-words --consonants ptkbdgmnŋsʃfvlr --vowels aeiou \
    --pattern "(X)VN"
```

## Named Sets `$name`

Named sets let you define custom groups of phonemes (including consonant clusters) in the JSON sketch and reference them
by name in patterns.
This is essential for phonotactic constraints where the set of allowed segments differs by position -- for example,
onsets allowing clusters like /st/ or /pl/ while codas allow /ks/ or /nd/.

### Defining sets in JSON

Add a `"sets"` field to your sketch.
Each key is a set name (referenced as `$<name>` in patterns), and each value is an array of IPA strings:

```json
{
  "consonants": "pbtdkgmnŋfvszhrl",
  "vowels": "aeiou",
  "sets": {
    "O": ["p", "b", "t", "d", "st", "sp", "tr", "pl"],
    "K": ["p", "t", "k", "m", "n", "ŋ", "nd", "ks"],
    "longer": ["ŋ", "d", "b"]
  },
  "word_classes": {
    "word": {
      "patterns": ["$OV$K", "$OV", "V$K", "V${longer}"]
    }
  }
}
```

> The `"word_classes"` supports different types of words.
> When you get to [grammar](./grammar.md), you will see how these are used.

Each entry in the array can be:

* A single segment: `"p"`, `"ŋ"`
* A cluster (multi-segment sequence): `"st"`, `"pl"`, `"nd"`
* A weighted entry: `{"value": "st", "weight": 5}` -- plain strings get weight 1 when any entry has an explicit weight

### Using sets in patterns

Reference a named set with `$` followed by the set name.
For single-character names, use `$O` directly.
For multi-character names, use curly braces: `${onset}`.

| Pattern                | Meaning                                                  |
|:-----------------------|:---------------------------------------------------------|
| `$OV$K`                | Onset from set O, then vowel, then coda from set K      |
| `${onset}V${coda}`     | Same, using multi-character set names                    |
| `$OV`                  | Onset from set O, then vowel (open syllable)             |
| `($O)V$K`              | Optional onset from set O, then vowel, then coda         |
| `(${onset})V${coda}`   | Same with braces                                         |

### Onset and coda restrictions

Named sets are the primary way to express positional constraints.
For example, English allows /st/ as an onset cluster but not as a coda, while /ks/ is a valid coda but not an onset:

```json
"sets": {
  "O": ["n", "t", "s", "st", "sp", "sk", "tɹ", "pɹ", "pl", "bl", "kl"],
  "K": ["n", "t", "s", "k", "ŋ", "nd", "nt", "ns", "ks", "lz", "lk"]
}
```

### Weighted sets

If any entry in a set uses the weighted form, all entries participate in weighted random selection.
Plain strings receive weight 1:

```json
"O": ["p", "t", {"value": "st", "weight": 5}]
```

Here, "st" is five times more likely to be selected than "p" or "t".
