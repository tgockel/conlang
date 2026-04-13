# Grammar

Word classes are the core way to define patterns in a language sketch.
Each class groups a set of [phonotactic patterns](phonotactics.md) and an optional [lexicon](#lexicons) under a name.
Without further structure, sentence generation picks random words from all classes and lines them up.
Adding **sentence templates** gives sentences recognizable structure: "the big cat sat on a mat" works because
determiners, nouns, verbs, adjectives, and prepositions each fill a distinct role.

## Word Classes

A word class groups a set of [phonotactic patterns](phonotactics.md) and an optional [lexicon](#lexicons) under a name.
The names are up to you, but common abbreviations from linguistics include:

| Name   | Short for   | Examples (English)          |
|:-------|:------------|:----------------------------|
| `det`  | determiner  | the, a, this, some          |
| `noun` | noun        | cat, idea, river            |
| `verb` | verb        | run, think, become          |
| `adj`  | adjective   | big, red, careful           |
| `adv`  | adverb      | quickly, very, often        |
| `prep` | preposition | in, on, with, from          |
| `pro`  | pronoun     | I, she, they, it            |
| `conj` | conjunction | and, but, or                |

These names have no special meaning to the tool -- they are simply keys in a map.
You could call them `a`, `b`, `c` if you wanted, but descriptive names make your grammar templates easier to read.
If your sketch has only one kind of word (no grammar), a single class named `"word"` works fine.

Define them in the `"word_classes"` field of your JSON sketch:

```json
{
  "word_classes": {
    "det": {
      "patterns": [
        {"value": "$OV", "weight": 10},
        {"value": "V",   "weight": 5}
      ],
      "lexicon": { "generate": { "size": 8 } }
    },
    "noun": {
      "patterns": [
        {"value": "$OV$K",       "weight": 30},
        {"value": "$OV$K $OV",   "weight": 6},
        {"value": "$OV $OV$K",   "weight": 4},
        {"value": "$OV$K $OV$K", "weight": 3}
      ],
      "lexicon": { "generate": { "size": 120 } }
    },
    "verb": {
      "patterns": [
        {"value": "$OV$K",     "weight": 25},
        {"value": "$OV",       "weight": 15},
        {"value": "$OV$K $OV", "weight": 5}
      ],
      "lexicon": { "generate": { "size": 80 } }
    }
  }
}
```

Each class has its own `"patterns"` array, which uses the same pattern language described in
[phonotactics](phonotactics.md) -- `C`, `V`, named sets, brackets, optional segments, and weights all work the same way.
Patterns can also be plain strings without weights:

The `"lexicon"` field is optional.
When present, it pre-generates a fixed vocabulary for that class and samples from it with a
[Zipfian frequency distribution](frequencies.md#rule-interference), the same way the top-level lexicon works.
When absent, each word is generated fresh from the patterns.

### Designing Word Classes

The key insight is that natural languages have two broad categories of words:

**Function words** (determiners, prepositions, pronouns, conjunctions) are short, few in number, and extremely frequent.
Give them simple patterns like `CV` or `V` and small lexicons (5--15 words).

**Content words** (nouns, verbs, adjectives, adverbs) are longer, more varied, and individually less frequent.
Give them patterns with codas and multiple syllables, and larger lexicons (40--200 words).

This asymmetry creates the rhythmic alternation that makes sentences feel like language rather than random sound.

## Sentence Templates

Templates define the structure of generated sentences.
Each template is a space-separated sequence of word class names:

```json
{
  "grammar": [
    {"value": "det noun verb det noun",     "weight": 20},
    {"value": "det noun verb",              "weight": 15},
    {"value": "pro verb det noun",          "weight": 12},
    {"value": "det adj noun verb det noun", "weight": 8},
    {"value": "det noun verb adj",          "weight": 5}
  ]
}
```

When generating a sentence, the system picks a template (using the same weighted random selection as everywhere else),
then generates one word per slot from the corresponding class.
The template determines sentence length -- there is no separate word-count range.

Every class name used in a template must be defined in `"word_classes"`.
If a template references an undefined class, the sketch will fail to load with an error.

For the lazy, templates can be plain strings and the generator will pick them with equal weight:

```json
{
  "grammar": ["det noun verb", "noun verb det noun"]
}
```

### Requirements

`"word_classes"` is required in every sketch -- it is the only way to define patterns.
`"grammar"` is optional.

When grammar is present, `generate-sentences` picks a template and fills each slot from the matching word class.
When grammar is absent, `generate-sentences` draws each word from a randomly chosen class, with a word count from
the `"sentence"` configuration (or the default 3--8).

The `generate-words` command always generates individual words from word classes.
Use `--class <name>` to restrict output to a single class, or omit it to draw from all classes.

## Lexicons

Each word class can have its own lexicon, independent of the others.
When a class has a lexicon, generation samples from the pre-built vocabulary rather than creating a new word each time.
This means the same "words" recur naturally, with common words appearing more often than rare ones (Zipfian distribution).

Within a single sentence, the same word from the same class will not appear consecutively.
Different classes track their repeats independently.

## Example

Putting it all together, here is a minimal sketch with word classes:

```json
{
  "consonants": "pbtdkgmnŋsʃlr",
  "vowels": "aeiou",
  "sets": {
    "O": ["p", "t", "k", "b", "d", "g", "st", "pl"],
    "K": ["p", "t", "k", "m", "n", "ŋ", "nd", "ks"]
  },
  "word_classes": {
    "det":  { "patterns": ["CV", "V"], "lexicon": { "generate": { "size": 6 } } },
    "noun": { "patterns": ["$OV$K", "$OV $OV$K"], "lexicon": { "generate": { "size": 50 } } },
    "verb": { "patterns": ["$OV$K", "$OV"], "lexicon": { "generate": { "size": 30 } } }
  },
  "grammar": [
    {"value": "det noun verb det noun", "weight": 20},
    {"value": "det noun verb",          "weight": 15},
    {"value": "noun verb det noun",     "weight": 10}
  ]
}
```

```sh
conlang generate-sentences --config sketch.json
conlang generate-words --config sketch.json --class noun
```

The output will have short determiners (`ta`, `o`, `ke`) alternating with longer nouns and verbs (`plokŋ`, `sta.dend`),
giving each sentence a recognizable shape.
