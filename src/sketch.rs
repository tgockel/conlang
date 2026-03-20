//! # Sketch
//!
//! A language sketch is a JSON description of a constructed language's phonological
//! system: its phoneme inventory, frequency distributions, and syllable patterns.

use crate::phone;
use serde::Deserialize;
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

/// A language sketch loaded from JSON.
#[derive(Debug, Deserialize)]
pub struct Sketch {
    pub consonants: Option<PhonemeSet>,
    pub vowels: Option<PhonemeSet>,
    pub non_pulmonics: Option<String>,
    pub others: Option<String>,
    pub patterns: Vec<String>,
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
    pub patterns: Vec<String>,
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

        Ok(Resolved {
            inventory,
            consonant_weights,
            vowel_weights,
            patterns: self.patterns,
        })
    }
}

fn resolve_consonant_set(
    set: PhonemeSet,
) -> Result<(Vec<phone::Segment>, Option<Vec<u32>>), SketchError> {
    match set {
        PhonemeSet::Simple(s) => {
            let segs = parse_consonant_string(&s)?;
            Ok((segs, None))
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
            Ok((segs, None))
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
fn cosine_weights(count: usize, a: f64) -> Vec<u32> {
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
        assert!(resolved.consonant_weights.is_none());
        assert!(resolved.vowel_weights.is_none());
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
        assert!(resolved.consonant_weights.is_none());
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
}
