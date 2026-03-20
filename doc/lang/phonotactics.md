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
conlang generate-syllables --consonants ptkmnsl --vowels aeiou --pattern "CV"
```

A language with CVC syllables and optional onsets:

```sh
conlang generate-syllables --consonants ptkbdgmnŋsʃlr --vowels aeiou \
    --pattern "(C)VC"
```

Two-syllable words with specific initial consonants:

```sh
conlang generate-syllables --consonants ptkbdgmnŋsʃlr --vowels aeiou \
    --pattern "[ptkbdg]VC CV"
```

Nasal-only codas with optional fricative onsets:

```sh
conlang generate-syllables --consonants ptkbdgmnŋsʃfvlr --vowels aeiou \
    --pattern "(X)VN"
```
