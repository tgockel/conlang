# Morphology

Morphology adds internal structure to generated words through affixation.
Prefixes and suffixes are defined as patterns that probabilistically attach to words of certain classes.
This makes words feel derived rather than invented from whole cloth.

## Configuration

Add a `morphology` section to your config JSON:

```json
{
  "morphology": {
    "decay": 0.5,
    "suffixes": [
      {
        "pattern": "V$K",
        "applies_to": ["verb"],
        "probability": 0.3,
        "label": "past tense"
      },
      {
        "pattern": "C",
        "applies_to": ["noun"],
        "probability": 0.35,
        "label": "plural"
      }
    ],
    "prefixes": [
      {
        "pattern": "CV",
        "applies_to": ["verb", "adj"],
        "probability": 0.1,
        "label": "negation"
      }
    ]
  }
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `decay` | number | No | Probability multiplier for each subsequent affix of the same type (prefix or suffix). Default `0.5`. Must be between 0.0 and 1.0. |
| `suffixes` | array | No | List of suffix definitions. |
| `prefixes` | array | No | List of prefix definitions. |

### Affix Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pattern` | string | Yes | A phonotactic pattern using the same [pattern syntax](./phonotactics.md) as word classes. Space-separated syllables are supported for multi-syllable affixes. |
| `applies_to` | array | No | Word classes this affix can attach to. If absent, applies to all classes. |
| `probability` | number | Yes | Probability of attachment, between 0.0 and 1.0. |
| `label` | string | No | Documentary label (e.g. "plural", "past tense"). Has no effect on generation. |

## Behavior

### Application Order

Morphology runs **after** word generation but **before** [stress assignment](stress.md).
This means stress is computed over the full derived form including affixes, matching how natural languages work (e.g., English "PHOto" vs "phoTOgraphy").

### Decay

When multiple affixes of the same type (prefix or suffix) are defined, each subsequent attachment multiplies its probability by the decay factor.
For example, with `decay: 0.5` and two suffixes both at `probability: 0.3`:

- First suffix: effective probability = 0.3
- If first attached, second suffix: effective probability = 0.3 * 0.5 = 0.15
- If both attached, a hypothetical third: effective probability = 0.3 * 0.25 = 0.075

This prevents runaway stacking while still allowing occasional multi-affix words.

### Lexicon Integration

When a word class uses a pre-generated lexicon, affixes are baked into the lexicon entries at generation time.
This means each lexicon word has its affixes fixed, maintaining consistency when the same word is reused.

### Class Filtering

If `applies_to` is specified, the affix only applies to words of those classes.
If `applies_to` is absent, the affix applies to all word classes.
When no word classes are in use, all affixes apply to all words.
