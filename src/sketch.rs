//! # Sketch
//!
//! A language sketch is a JSON description of a constructed language's phonological
//! system: its phoneme inventory, frequency distributions, and syllable patterns.

use crate::phone;
use serde::Deserialize;
use smallvec::SmallVec;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SketchError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid phoneme: {0}")]
    Phoneme(#[from] phone::ParseError),
    #[error("{0}")]
    Validation(String),
}

/// A single entry in a named set: either a plain IPA string or one with an explicit weight.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NamedSetEntry {
    Simple(String),
    Weighted { value: String, weight: u32 },
}

/// A resolved named set: each choice is a sequence of segments (to support clusters).
pub struct NamedSet {
    pub choices: Vec<SmallVec<[phone::Segment; 2]>>,
    pub weights: Option<Vec<u32>>,
}

/// Configuration for sentence generation.
#[derive(Debug, Deserialize)]
pub struct SentenceConfig {
    /// `[min, max]` inclusive range for word count per sentence.
    pub words: [u32; 2],
}

/// Configuration for the lexicon.
#[derive(Debug, Deserialize)]
pub struct LexiconConfig {
    /// Pre-generate a vocabulary from the phonotactic patterns.
    pub generate: Option<LexiconGenerateConfig>,
}

/// Configuration for pre-generating a reusable vocabulary.
#[derive(Debug, Deserialize)]
pub struct LexiconGenerateConfig {
    /// Number of distinct words to pre-generate.
    pub size: usize,
}

/// A language sketch loaded from JSON.
#[derive(Debug, Deserialize)]
pub struct Sketch {
    pub consonants: Option<PhonemeSet>,
    pub vowels: Option<PhonemeSet>,
    pub non_pulmonics: Option<String>,
    pub others: Option<String>,
    pub sets: Option<HashMap<String, Vec<NamedSetEntry>>>,
    pub patterns: Vec<NamedSetEntry>,
    pub sentence: Option<SentenceConfig>,
    pub lexicon: Option<LexiconConfig>,
}

/// Three ways to specify a set of phonemes, matching the documented JSON formats.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PhonemeSet {
    /// A plain string of IPA characters, e.g. `"pbtdkg"`.
    Simple(String),
    /// An object with a `values` string and a `distribution` specification.
    WithDistribution {
        values: String,
        distribution: Distribution,
    },
    /// An array of individually weighted phonemes.
    CustomWeighted(Vec<WeightedPhoneme>),
}

#[derive(Debug, Deserialize)]
pub struct Distribution {
    pub curve: String,
    pub a: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct WeightedPhoneme {
    pub value: String,
    pub weight: u32,
}

/// The result of resolving a sketch into domain types.
pub struct Resolved {
    pub inventory: phone::Inventory,
    pub consonant_weights: Option<Vec<u32>>,
    pub vowel_weights: Option<Vec<u32>>,
    pub named_sets: HashMap<String, NamedSet>,
    pub patterns: Vec<String>,
    pub pattern_weights: Option<Vec<u32>>,
    pub sentence: Option<SentenceConfig>,
    pub lexicon: Option<LexiconGenerateConfig>,
}

impl Sketch {
    pub fn load(json: &str) -> Result<Resolved, SketchError> {
        let sketch: Sketch = serde_json::from_str(json)?;
        sketch.resolve()
    }

    pub fn resolve(self) -> Result<Resolved, SketchError> {
        if self.patterns.is_empty() {
            return Err(SketchError::Validation("patterns must not be empty".into()));
        }

        if let Some(ref sc) = self.sentence {
            if sc.words[0] == 0 {
                return Err(SketchError::Validation(
                    "sentence.words minimum must be at least 1".into(),
                ));
            }
            if sc.words[0] > sc.words[1] {
                return Err(SketchError::Validation(
                    "sentence.words minimum must not exceed maximum".into(),
                ));
            }
        }

        let (consonants, consonant_weights) = match self.consonants {
            Some(set) => resolve_consonant_set(set)?,
            None => (Vec::new(), None),
        };

        let (vowels, vowel_weights) = match self.vowels {
            Some(set) => resolve_vowel_set(set)?,
            None => (Vec::new(), None),
        };

        let mut all_consonants = consonants;

        if let Some(np_str) = self.non_pulmonics {
            for ch in np_str.chars() {
                let np = phone::NonPulmonicConsonant::try_from(ch)?;
                all_consonants.push(phone::Segment::from(np));
            }
        }

        let inventory = phone::Inventory::from_segments(all_consonants, vowels);

        let named_sets = match self.sets {
            Some(sets) => resolve_named_sets(sets)?,
            None => HashMap::new(),
        };

        let has_weights = self
            .patterns
            .iter()
            .any(|e| matches!(e, NamedSetEntry::Weighted { .. }));
        let mut pattern_strings = Vec::with_capacity(self.patterns.len());
        let mut pattern_weights_vec = if has_weights {
            Some(Vec::with_capacity(self.patterns.len()))
        } else {
            None
        };
        for entry in self.patterns {
            let (value, weight) = match entry {
                NamedSetEntry::Simple(s) => (s, 1),
                NamedSetEntry::Weighted { value, weight } => (value, weight),
            };
            pattern_strings.push(value);
            if let Some(ref mut w) = pattern_weights_vec {
                w.push(weight);
            }
        }

        Ok(Resolved {
            inventory,
            consonant_weights,
            vowel_weights,
            named_sets,
            patterns: pattern_strings,
            pattern_weights: pattern_weights_vec,
            sentence: self.sentence,
            lexicon: self.lexicon.and_then(|l| l.generate),
        })
    }
}

fn resolve_consonant_set(
    set: PhonemeSet,
) -> Result<(Vec<phone::Segment>, Option<Vec<u32>>), SketchError> {
    match set {
        PhonemeSet::Simple(s) => {
            let segs = parse_consonant_string(&s)?;
            let weights = cosine_weights(segs.len(), 0.0);
            Ok((segs, Some(weights)))
        }
        PhonemeSet::WithDistribution { values, distribution } => {
            let segs = parse_consonant_string(&values)?;
            let weights = compute_weights(segs.len(), &distribution)?;
            Ok((segs, Some(weights)))
        }
        PhonemeSet::CustomWeighted(entries) => {
            let mut segs = Vec::with_capacity(entries.len());
            let mut weights = Vec::with_capacity(entries.len());
            for entry in entries {
                let (seg, rest) = phone::Segment::parse_ipa(&entry.value)?;
                if !rest.is_empty() {
                    return Err(SketchError::Validation(format!(
                        "trailing characters in phoneme value: {:?}",
                        entry.value
                    )));
                }
                segs.push(seg);
                weights.push(entry.weight);
            }
            Ok((segs, Some(weights)))
        }
    }
}

fn resolve_vowel_set(
    set: PhonemeSet,
) -> Result<(Vec<phone::Segment>, Option<Vec<u32>>), SketchError> {
    match set {
        PhonemeSet::Simple(s) => {
            let segs = parse_vowel_string(&s)?;
            let weights = cosine_weights(segs.len(), 0.0);
            Ok((segs, Some(weights)))
        }
        PhonemeSet::WithDistribution { values, distribution } => {
            let segs = parse_vowel_string(&values)?;
            let weights = compute_weights(segs.len(), &distribution)?;
            Ok((segs, Some(weights)))
        }
        PhonemeSet::CustomWeighted(entries) => {
            let mut segs = Vec::with_capacity(entries.len());
            let mut weights = Vec::with_capacity(entries.len());
            for entry in entries {
                let (seg, rest) = phone::Segment::parse_ipa(&entry.value)?;
                if !rest.is_empty() {
                    return Err(SketchError::Validation(format!(
                        "trailing characters in phoneme value: {:?}",
                        entry.value
                    )));
                }
                segs.push(seg);
                weights.push(entry.weight);
            }
            Ok((segs, Some(weights)))
        }
    }
}

fn parse_consonant_string(s: &str) -> Result<Vec<phone::Segment>, phone::ParseError> {
    let mut segs = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let c = phone::Consonant::try_from(ch)?;
        segs.push(phone::Segment::from(c));
    }
    Ok(segs)
}

fn parse_vowel_string(s: &str) -> Result<Vec<phone::Segment>, phone::ParseError> {
    let mut segs = Vec::with_capacity(s.len());
    for ch in s.chars() {
        let v = phone::Vowel::try_from(ch)?;
        segs.push(phone::Segment::from(v));
    }
    Ok(segs)
}

fn compute_weights(count: usize, dist: &Distribution) -> Result<Vec<u32>, SketchError> {
    match dist.curve.as_str() {
        "cosine" => {
            let a = dist.a.unwrap_or(0.0);
            Ok(cosine_weights(count, a))
        }
        "uniform" => Ok(vec![1; count]),
        other => Err(SketchError::Validation(format!(
            "unknown distribution curve: {other:?}"
        ))),
    }
}

/// Compute weights from the cosine distribution documented in frequencies.md.
///
/// For k phonemes, the weight of the nth is the integral of (cos(x) + a)
/// from n*pi/(2k) to (n+1)*pi/(2k).
pub fn cosine_weights(count: usize, a: f64) -> Vec<u32> {
    let k = count as f64;
    let scale = 10_000.0;
    (0..count)
        .map(|n| {
            let n = n as f64;
            let lo = n * std::f64::consts::FRAC_PI_2 / k;
            let hi = (n + 1.0) * std::f64::consts::FRAC_PI_2 / k;
            let weight = hi.sin() - lo.sin() + a * (hi - lo);
            (weight * scale).round() as u32
        })
        .collect()
}

fn parse_segment_sequence(s: &str) -> Result<SmallVec<[phone::Segment; 2]>, phone::ParseError> {
    let mut segs = SmallVec::new();
    let mut remaining = s;
    while !remaining.is_empty() {
        let (seg, rest) = phone::Segment::parse_ipa(remaining)?;
        segs.push(seg);
        remaining = rest;
    }
    Ok(segs)
}

fn resolve_named_sets(
    sets: HashMap<String, Vec<NamedSetEntry>>,
) -> Result<HashMap<String, NamedSet>, SketchError> {
    let mut result = HashMap::new();
    for (name, entries) in sets {
        if entries.is_empty() {
            return Err(SketchError::Validation(format!(
                "named set {name:?} is empty"
            )));
        }
        let has_weights = entries
            .iter()
            .any(|e| matches!(e, NamedSetEntry::Weighted { .. }));
        let mut choices = Vec::with_capacity(entries.len());
        let mut weights = if has_weights {
            Some(Vec::with_capacity(entries.len()))
        } else {
            None
        };
        for entry in entries {
            let (value, weight) = match entry {
                NamedSetEntry::Simple(s) => (s, 1),
                NamedSetEntry::Weighted { value, weight } => (value, weight),
            };
            let segs = parse_segment_sequence(&value)?;
            choices.push(segs);
            if let Some(ref mut w) = weights {
                w.push(weight);
            }
        }
        result.insert(name, NamedSet { choices, weights });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_sketch() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 3);
        assert_eq!(resolved.inventory.vowels().len(), 3);
        let cw = resolved.consonant_weights.unwrap();
        assert_eq!(cw.len(), 3);
        let vw = resolved.vowel_weights.unwrap();
        assert_eq!(vw.len(), 3);
        assert_eq!(resolved.patterns, vec!["CVC"]);
    }

    #[test]
    fn cosine_distribution() {
        let json = r#"{
            "consonants": {
                "values": "pbɣʂʐɟkg",
                "distribution": { "curve": "cosine" }
            },
            "vowels": "aiu",
            "patterns": ["CVC"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let w = resolved.consonant_weights.unwrap();
        assert_eq!(w.len(), 8);
        // First element should be ~19.5% of total, last ~1.9%
        let total: u32 = w.iter().sum();
        let first_pct = w[0] as f64 / total as f64;
        let last_pct = w[7] as f64 / total as f64;
        assert!((first_pct - 0.195).abs() < 0.01, "first: {first_pct}");
        assert!((last_pct - 0.019).abs() < 0.01, "last: {last_pct}");
    }

    #[test]
    fn custom_weighted() {
        let json = r#"{
            "consonants": [
                { "value": "p", "weight": 30 },
                { "value": "t", "weight": 20 },
                { "value": "k", "weight": 10 }
            ],
            "vowels": "aiu",
            "patterns": ["CVC"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 3);
        let w = resolved.consonant_weights.unwrap();
        assert_eq!(w, vec![30, 20, 10]);
    }

    #[test]
    fn mixed_formats() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": [
                { "value": "a", "weight": 50 },
                { "value": "i", "weight": 30 }
            ],
            "patterns": ["CV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.consonant_weights.unwrap().len(), 3);
        assert_eq!(resolved.vowel_weights.unwrap(), vec![50, 30]);
    }

    #[test]
    fn with_non_pulmonics() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "ai",
            "non_pulmonics": "ɓɗ",
            "patterns": ["CVC"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        // 3 consonants + 2 non-pulmonics
        assert_eq!(resolved.inventory.consonants().len(), 5);
    }

    #[test]
    fn empty_patterns_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": []
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn cosine_with_parameter_a() {
        let json = r#"{
            "consonants": {
                "values": "pbɣʂʐɟkgqʕβt",
                "distribution": { "curve": "cosine", "a": 0.2 }
            },
            "vowels": "a",
            "patterns": ["CV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let w = resolved.consonant_weights.unwrap();
        assert_eq!(w.len(), 12);
        // With a=0.2, the distribution should be less extreme than a=0
        let total: u32 = w.iter().sum();
        let first_pct = w[0] as f64 / total as f64;
        let last_pct = w[11] as f64 / total as f64;
        // From the documented table: first ~11.9%, last ~2.6%
        assert!((first_pct - 0.119).abs() < 0.01, "first: {first_pct}");
        assert!((last_pct - 0.026).abs() < 0.01, "last: {last_pct}");
    }

    #[test]
    fn custom_weighted_affricate() {
        let json = r#"{
            "consonants": [
                { "value": "t͡ʃ", "weight": 20 },
                { "value": "d͡ʒ", "weight": 10 }
            ],
            "vowels": "a",
            "patterns": ["CV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 2);
        // Both should be compound segments
        assert!(resolved.inventory.consonants()[0].manner() == Some(phone::Manner::Affricate));
    }

    #[test]
    fn named_set_single_segments() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "sets": {
                "O": ["p", "t", "k"]
            },
            "patterns": ["$OV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let onset = &resolved.named_sets["O"];
        assert_eq!(onset.choices.len(), 3);
        assert!(onset.weights.is_none());
    }

    #[test]
    fn named_set_with_clusters() {
        let json = r#"{
            "consonants": "ptksl",
            "vowels": "a",
            "sets": {
                "O": ["p", "st", "pl"]
            },
            "patterns": ["$OV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let onset = &resolved.named_sets["O"];
        assert_eq!(onset.choices.len(), 3);
        // "st" should have 2 segments
        assert_eq!(onset.choices[1].len(), 2);
    }

    #[test]
    fn named_set_weighted() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "a",
            "sets": {
                "O": ["p", {"value": "t", "weight": 5}]
            },
            "patterns": ["$OV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let onset = &resolved.named_sets["O"];
        assert_eq!(onset.choices.len(), 2);
        let w = onset.weights.as_ref().unwrap();
        assert_eq!(w, &[1, 5]);
    }

    #[test]
    fn named_set_empty_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "a",
            "sets": {
                "O": []
            },
            "patterns": ["$OV"]
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn sentence_config_loads() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"],
            "sentence": { "words": [3, 8] }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let sc = resolved.sentence.unwrap();
        assert_eq!(sc.words, [3, 8]);
    }

    #[test]
    fn sentence_config_absent() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert!(resolved.sentence.is_none());
    }

    #[test]
    fn sentence_config_min_zero_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"],
            "sentence": { "words": [0, 5] }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn sentence_config_min_exceeds_max_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"],
            "sentence": { "words": [8, 3] }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn weighted_patterns() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": [
                {"value": "CVC", "weight": 40},
                {"value": "CV", "weight": 10}
            ]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.patterns, vec!["CVC", "CV"]);
        assert_eq!(resolved.pattern_weights, Some(vec![40, 10]));
    }

    #[test]
    fn mixed_patterns_plain_and_weighted() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC", {"value": "CV", "weight": 5}]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.patterns, vec!["CVC", "CV"]);
        assert_eq!(resolved.pattern_weights, Some(vec![1, 5]));
    }

    #[test]
    fn plain_patterns_no_weights() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC", "CV"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.patterns, vec!["CVC", "CV"]);
        assert!(resolved.pattern_weights.is_none());
    }

    #[test]
    fn lexicon_config_loads() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"],
            "lexicon": { "generate": { "size": 100 } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let lex = resolved.lexicon.unwrap();
        assert_eq!(lex.size, 100);
    }

    #[test]
    fn lexicon_config_absent() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "patterns": ["CVC"]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert!(resolved.lexicon.is_none());
    }
}
