# Phonemes

> **NOTE**
>
> This page acts as a quick introduction to phonetics.
> If you are already familiar with phonetics, feel free to skip down to [inventory](#inventory).
> This page is also the reference for the predefined phonetic categories (e.g.: `Ḍ`, `Q`, `I`, etc) that are useful for
> generation based on photological constraints, so you will likely be coming back to this page.

A _phoneme_ is the basic unit of sound in speech.
These are captured in the _International Phonetic Alphabet_, which provides a mapping of these sounds into symbols.
In examples, IPA is written `/laɪk ðɪs/` ("like this") -- monospaced and surrounded by `/`s.

Reading IPA does not come naturally, partially due to the abundance of funny symbols like `ɤ`, `ɮ`, and `ɰ`.
You might be familiar with `e`, but what are `ɘ` and `ə` supposed to mean?
Searching for these is also a challenge, since `ʔ` will turn up that `ʔ` is the symbol for an unaspirated glottal stop,
which is not helpful until you know what that means.
Another hurdle is that symbols sometimes do not align with their written-language meanings; for example, the IPA `/fit/`
is pronounced like "feet," not "fit" as the spelling might seem.

The best way to familiarize yourself with what these symbols sound like is to use the [IPA Chart][ipa-chart] and click
the symbols to listen to them.
Note that vowels are pronounced directly, while consonants get a vowel sound to accentuate it (try pronoucing `/d/`) on
its own).
After getting a baseline, look up some words in the [dictionary][dictionary] and show their IPAs.
Copy and paste these into the [IPA Reader][ipa-reader] and you can start deconstructing words to see what happens.
You can also do this with `conlang speak ...`.

## IPA

### Consonants

Consonants are categorized based on the _place_ and _manner_ of articulation.
The _place of articulation_ refers to where in the vocal tract the sound comes from; a `/p/` is bilabial, meaning it
comes from your lips pressing together, whereas a `/k/` is velar, coming from the back of the mouth.
The _manner of articulation_ refers to how the sound is made.
This is easiest to distinguish when looking at sounds with the same place; `/t/`, `/n/`, and `/s/` are all alveolar
(tongue pressed on the top of your front teeth), but the plosive (full stop), nasal (air goes into the nose), and
fricative (restricted hissing) manners make different sounds.

<table class="compressed">
  <thead>
    <tr>
      <th /><th class="vlabel"><span>Place</span></th>
      <th colspan="2" class="vlabel"><span>Bilabial</span></th>
      <th colspan="2" class="vlabel"><span>Labiodental</span></th>
      <th colspan="2" class="vlabel"><span>Dental</span></th>
      <th colspan="2" class="vlabel"><span>Alveolar</span></th>
      <th colspan="2" class="vlabel"><span>Post alveolar</span></th>
      <th colspan="2" class="vlabel"><span>Retroflex</span></th>
      <th colspan="2" class="vlabel"><span>Palatal</span></th>
      <th colspan="2" class="vlabel"><span>Velar</span></th>
      <th colspan="2" class="vlabel"><span>Uvular</span></th>
      <th colspan="2" class="vlabel"><span>Pharyngeal</span></th>
      <th colspan="2" class="vlabel"><span>Glottal</span></th>
    </tr>
    <tr style="text-align: center">
      <th colspan="2" style="text-align: left">Manner</th>
      <th colspan="2">M</th>
      <th colspan="2">L</th>
      <th colspan="6">D</th>
      <th colspan="2">Ḍ</th>
      <th colspan="2">J</th>
      <th colspan="2">G</th>
      <th colspan="2">Q</th>
      <th colspan="4">H</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th>Plosive</th><th>P</th>
      <td><code class="hljs">p</code></td><td><code class="hljs">b</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td><code class="hljs">t</code></td><td />
      <td><code class="hljs">d</code></td><td />
      <td><code class="hljs">ʈ</code></td><td><code class="hljs">ɖ</code></td>
      <td><code class="hljs">c</code></td><td><code class="hljs">ɟ</code></td>
      <td><code class="hljs">k</code></td><td><code class="hljs">g</code></td>
      <td><code class="hljs">q</code></td><td><code class="hljs">ɢ</code></td>
      <td colspan="2" />
      <td><code class="hljs">ʔ</code></td><td />
    </tr>
    <tr>
      <th>Nasal</th><th>N</th>
      <td /><td><code class="hljs">m</code></td>
      <td /><td><code class="hljs">ɱ</code></td>
      <td colspan="2" />
      <td><code class="hljs">n</code></td><td />
      <td colspan="2" />
      <td /><td><code class="hljs">ɳ</code></td>
      <td /><td><code class="hljs">ŋ</code></td>
      <td /><td><code class="hljs">ɴ</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
    </tr>
    <tr>
      <th>Trill</th><th rowspan="2">T</th>
      <td /><td><code class="hljs">ʙ</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td><code class="hljs">r</code></td><td />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td /><td><code class="hljs">ʀ</code></td>
      <td colspan="2" />
      <td colspan="2" />
    </tr>
    <tr>
      <th>Tap/Flap</th>
      <td colspan="2" />
      <td /><td><code class="hljs">ⱱ</code></td>
      <td colspan="2" />
      <td><code class="hljs">ɾ</code></td><td />
      <td colspan="2" />
      <td /><td><code class="hljs">ɽ</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
    </tr>
    <tr>
      <th>Fricative</th><th rowspan="2">X</th>
      <td><code class="hljs">ɸ</code></td><td><code class="hljs">β</code></td>
      <td><code class="hljs">f</code></td><td><code class="hljs">v</code></td>
      <td><code class="hljs">θ</code></td><td><code class="hljs">ð</code></td>
      <td><code class="hljs">s</code></td><td><code class="hljs">z</code></td>
      <td><code class="hljs">ʃ</code></td><td><code class="hljs">ʒ</code></td>
      <td><code class="hljs">ʂ</code></td><td><code class="hljs">ʐ</code></td>
      <td><code class="hljs">ç</code></td><td><code class="hljs">ʝ</code></td>
      <td><code class="hljs">x</code></td><td><code class="hljs">ɣ</code></td>
      <td><code class="hljs">χ</code></td><td><code class="hljs">ʁ</code></td>
      <td><code class="hljs">ħ</code></td><td><code class="hljs">ʕ</code></td>
      <td><code class="hljs">h</code></td><td><code class="hljs">ɦ</code></td>
    </tr>
    <tr>
      <th>Lateral Fricative</th>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td><code class="hljs">ɬ</code></td><td><code class="hljs">ɮ</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
    </tr>
    <tr>
      <th>Approximant</th><th rowspan="2">R</th>
      <td colspan="2" />
      <td /><td><code class="hljs">ʋ</code></td>
      <td colspan="2" />
      <td /><td><code class="hljs">ɹ</code></td>
      <td colspan="2" />
      <td /><td><code class="hljs">ɻ</code></td>
      <td /><td><code class="hljs">j</code></td>
      <td /><td><code class="hljs">ɰ</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
    </tr>
    <tr>
      <th>Lateral Approximant</th>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
      <td /><td><code class="hljs">l</code></td>
      <td colspan="2" />
      <td /><td><code class="hljs">ɭ</code></td>
      <td /><td><code class="hljs">ʎ</code></td>
      <td /><td><code class="hljs">ʟ</code></td>
      <td colspan="2" />
      <td colspan="2" />
      <td colspan="2" />
    </tr>
  </tbody>
</table>

The boldface capital letters at the top of the columns and left of the rows represent the predefined phonetic categories
available for generation.
For example, **N** can be used for any of `m`, `ɱ`, `n`, `ɳ`, `ŋ`, `ɴ`.
Some of these categories span multiple places or manners of articulation.
**D** covers dental, alveolar, and post alveolar; **T** covers trill, tap, and flap; and **H** covers pharyngeal and
glottal.
These letters do not come from any official phonetic studies, but serve as convenience when writing
[phonological constraints](phonotactics.md).

### Vowels

|               | Front     |         | Mid     |       | Back    |
|:------------- |:---------:|:-------:|:-------:|:-----:|:-------:|
| Close         | `i` `y`   |         | `ɨ` `ʉ` |       | `ɯ` `u` |
|               |           | `ɪ` `ʏ` |         | `ʊ`   |         |
| Close-mid     | `e` `ø`   |         | `ɘ` `ɵ` |       | `ɤ` `o` |
|               |           |         | `ə`     |       |         |
| Open-mid      | `ɛ` `œ`   |         | `ɜ` `ɞ` |       | `ʌ` `ɔ` |
|               | `æ`       |         | `ɐ`     |       |         |
| Open          |           | `a` `ɶ` |         |       | `ɑ`     |

### Non-pulmonic Consonants

| Clicks                | Voiced Implosives |
|:--------------------- |:----------------- |
| `ʘ` Bilabial          | `ɓ` Bilabial      |
| `ǀ` Dental            | `ɗ` Dental        |
| `ǃ` Postalveoalar     | `ʄ` Palatal       |
| `ǂ` Palatoalveolar    | `ɠ` Velar         |
| `ǁ` Alveolar Lateral  | `ʛ` Uvular        |

### Other

## Inventory

Your _phonetic inventory_ is the set of available sounds which are native to that language.

The simplest representation of this inventory is a string of consonants and vowels.
In JSON, this is an object with a couple of keys.
The [non-pulmonic constants](#non-pulmonic-consonants) and [others](#other) are not required, but are shown here to
demonstrate their defaults.

```json
{
  "consonants": "pbɣʂʐɟkgqʕβt",
  "vowels": "ɘɵæiəɯɨɑɤyɛœaʉ",
  "non_pulmonics": "",
  "others": ""
}
```

[dictionary]: https://www.dictionary.com/
[ipa-chart]:  https://www.ipachart.com/
[ipa-reader]: http://ipa-reader.xyz/
