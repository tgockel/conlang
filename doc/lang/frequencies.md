# Frequencies

This page covers the frequencies of fragments that you find in [phonemes](phonemes.md) and
[phonological constraints](phonotactics.md).
When generating sounds and text, how frequently should certain things be used?
For example, if a word is being generated with consonant inventory `"pbɣʂʐɟkgqʕβt"` and the rule calls for a consonant,
how often should `/p/` be chosen vs `/b/` vs ... vs `/β/` vs `/t/`?
By default, generation follows the [sinusoidal](#sinusoidal) distribution pattern, but this can be customized.

## Distributions

### Uniform

The most basic frequency distribution is a uniform one.

### Sinusoidal

The sinusoidal distribution is based on [\\( \cos{x} |_{0}^{\pi/2} \\)](https://www.wolframalpha.com/input?i=graph+cos(x)+for+x+from+0+to+pi/2).
Given \\( k \\) possible elements, the starting position on the curve for the \\( n \\)th element is given by
\\( p(n) \\), with ending position at \\( p(n+1) \\):

\\[
  p(n) = \frac{n\pi}{2k}
\\]

The probability of choosing the \\( n \\)th element is given by the integral from \\( p(n) \\) to \\( p(n+1) \\):

\\[
  \int_{p(n)}^{p(n+1)} \cos x
  = \left[ \sin x \right]_{p(n)}^{p(n+1)}
\\]

As an example, the consonant inventory `"pbɣʂʐɟkg"` would have the distribution:

|  `p`  |  `b`  |  `ɣ`  |  `ʂ`  |  `ʐ`  |  `ɟ`  |  `k`  |  `g`  |
|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|:-----:|
| 19.5% | 18.7% | 17.2% | 15.1% | 12.4% |  9.2% |  5.6% |  1.9% |

It might seem extreme that `p` appears approximate 10 times more frequently than `g`, but most languages follow such
extreme patterns.
This is why Scrabble rewards you 10 times the points for playing a "Q" than an "S."

#### Customizing Cosine

The distribution \\( f \\) can be customized over its \\( a \\) parameter:

\\[
  f(x; a) = \cos x + a |_{0}^{\pi/2}
\\]

```json
{
  "consonants": {
    "values": "pbɣʂʐɟkgqʕβt",
    "distribution": {
      "curve": "cosine",
      "a": 0.2
    }
  }
}
```

This can even out the extremes of the later-appearing values by giving them a bit of extra weight.

| a → |  0.0  |  0.1  |  0.2  |  0.5  |  1.0  |  2.0 | 10.0 |
| ---:| -----:| -----:| -----:| -----:| -----:| ----:| ----:|
| `p` | 13.1% | 12.4% | 11.9% | 11.0% | 10.2% | 9.5% | 8.6% |
| `b` | 12.8% | 12.2% | 11.8% | 10.9% | 10.1% | 9.4% | 8.6% |
| `ɣ` | 12.4% | 11.8% | 11.4% | 10.6% |  9.9% | 9.3% | 8.6% |
| `ʂ` | 11.7% | 11.3% | 10.9% | 10.2% |  9.7% | 9.2% | 8.5% |
| `ʐ` | 10.9% | 10.5% | 10.3% |  9.8% |  9.3% | 8.9% | 8.5% |
| `ɟ` |  9.8% |  9.6% |  9.5% |  9.2% |  8.9% | 8.7% | 8.4% |
| `k` |  8.6% |  8.6% |  8.6% |  8.5% |  8.4% | 8.4% | 8.4% |
| `g` |  7.3% |  7.4% |  7.5% |  7.7% |  7.9% | 8.1% | 8.3% |
| `q` |  5.8% |  6.1% |  6.4% |  6.9% |  7.3% | 7.7% | 8.2% |
| `ʕ` |  4.2% |  4.8% |  5.2% |  6.0% |  6.7% | 7.3% | 8.1% |
| `β` |  2.6% |  3.3% |  3.9% |  5.1% |  6.1% | 6.9% | 8.0% |
| `t` |  0.9% |  1.9% |  2.6% |  4.1% |  5.4% | 6.5% | 7.9% |

In my experience, setting \\( a \\) to anything above \\( 0.2 \\) leads to fairly monotonous languages.
Natural languages have these rarely-used sounds in them.

### Custom

```json
{
  "consonants": [
    { "value": "ɴ", "weight": 30 },
    { "value": "f", "weight": 20 },
    { "value": "ʝ", "weight": 10 },
    { "value": "θ", "weight":  5 },
    { "value": "b", "weight":  5 }
  ]
}
```

## Rule Interference

Since the same phoneme is not allowed to be generated twice in a row, the constraint rules can interfere with the
observed frequencies in generated patterns.
This is most likely to occur in `VV` phonotactic patterns, but can occur in any pattern pulling from a repeating pool.

As an example, consider the vowel inventory:

```json
{
  "vowels": [
    { "value": "ɘ", "weight": 50 },
    { "value": "ɑ", "weight": 15 },
    { "value": "i", "weight":  5 }
  ]
}
```

The pattern `VV` will generate `/ɘɑ/` 3 times more than it will generate `/ɘi/` because the weights without `/ɘ/` mean
there is a 75%/25% distribution for the remainder.
What might be more disturbing is that `/ɑɘ/` will be generated 10 times more than `/ɑi/`.

This is not usually a problem, since there are usually enough sounds in your inventory to prevent anyone from noticing.
However, it can be an issue when languages have a limited inventory or cases where your phonological constraints are too
restrictive.
Note that these interact; there aren't that many nasals, so `VDN` is more restrictive if your consonant set only has
`/n/` and `/ɴ/`.
