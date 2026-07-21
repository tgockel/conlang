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

/// Stress assignment strategy for a word class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StressStrategy {
    Trochaic,
    Iambic,
    Penultimate,
    Final,
    None,
}

/// Top-level stress configuration.
#[derive(Debug, Deserialize)]
pub struct StressConfig {
    /// Default stress strategy for all word classes.
    pub default: StressStrategy,
    /// Whether to assign secondary stress on 3+ syllable words.
    #[serde(default)]
    pub secondary: bool,
}

/// Configuration for unstressed-vowel reduction.
#[derive(Debug, Deserialize)]
pub struct ReductionConfig {
    /// Probability (0.0–1.0) an unstressed syllable's vowel reduces.
    pub probability: f64,
    /// Single vowel every reduced vowel collapses to. Mutually exclusive with
    /// `targets`; exactly one of the two is required.
    pub target: Option<String>,
    /// Map from source vowels (each key a string of IPA vowels) to the single
    /// vowel they reduce to. Mutually exclusive with `target`.
    pub targets: Option<HashMap<String, String>>,
}

/// What reduced vowels turn into.
#[derive(Debug, Clone)]
pub enum ReductionTargets {
    /// Every vowel reduces to this single target.
    All(phone::Vowel),
    /// Only mapped vowels reduce, each to its mapped target.
    Map(HashMap<phone::Vowel, phone::Vowel>),
}

impl ReductionTargets {
    /// The vowel `v` reduces to, or `None` if `v` does not reduce.
    pub fn target_for(&self, v: phone::Vowel) -> Option<phone::Vowel> {
        match self {
            Self::All(t) => Some(*t),
            Self::Map(map) => map.get(&v).copied(),
        }
    }
}

/// Resolved vowel-reduction settings.
#[derive(Debug, Clone)]
pub struct ResolvedReduction {
    /// Probability (0.0–1.0) an unstressed syllable's vowel reduces.
    pub probability: f64,
    pub targets: ReductionTargets,
}

impl Default for ResolvedReduction {
    /// Reduction disabled: zero probability (the target is never consulted).
    fn default() -> Self {
        Self {
            probability: 0.0,
            targets: ReductionTargets::All(phone::Vowel::Schwa),
        }
    }
}

/// Configuration for morphological affixation.
#[derive(Debug, Deserialize)]
pub struct MorphologyConfig {
    /// Probability multiplier for each subsequent affix (default 0.5).
    pub decay: Option<f64>,
    pub suffixes: Option<Vec<AffixConfig>>,
    pub prefixes: Option<Vec<AffixConfig>>,
}

/// A single affix definition.
#[derive(Debug, Deserialize)]
pub struct AffixConfig {
    pub pattern: String,
    pub applies_to: Option<Vec<String>>,
    pub probability: f64,
    pub label: Option<String>,
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

/// Configuration for a single word class.
#[derive(Debug, Deserialize)]
pub struct WordClassConfig {
    pub patterns: Vec<NamedSetEntry>,
    pub lexicon: Option<LexiconConfig>,
    pub stress: Option<StressStrategy>,
}

/// A language sketch loaded from JSON.
#[derive(Debug, Deserialize)]
pub struct Sketch {
    pub consonants: Option<PhonemeSet>,
    pub vowels: Option<PhonemeSet>,
    pub non_pulmonics: Option<String>,
    pub others: Option<String>,
    pub sets: Option<HashMap<String, Vec<NamedSetEntry>>>,
    pub word_classes: HashMap<String, WordClassConfig>,
    pub sentence: Option<SentenceConfig>,
    pub grammar: Option<Vec<NamedSetEntry>>,
    pub stress: Option<StressConfig>,
    pub reduction: Option<ReductionConfig>,
    pub morphology: Option<MorphologyConfig>,
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

/// A resolved word class ready for generation.
pub struct ResolvedWordClass {
    pub patterns: Vec<String>,
    pub pattern_weights: Option<Vec<u32>>,
    pub lexicon: Option<LexiconGenerateConfig>,
    pub stress: Option<StressStrategy>,
    pub secondary_stress: bool,
    pub reduction: ResolvedReduction,
}

/// A resolved affix ready for generation.
pub struct ResolvedAffix {
    pub pattern: String,
    pub applies_to: Option<Vec<String>>,
    pub probability: f64,
}

/// Resolved morphology configuration.
pub struct ResolvedMorphology {
    pub decay: f64,
    pub prefixes: Vec<ResolvedAffix>,
    pub suffixes: Vec<ResolvedAffix>,
}

/// The result of resolving a sketch into domain types.
pub struct Resolved {
    pub inventory: phone::Inventory,
    pub consonant_weights: Option<Vec<u32>>,
    pub vowel_weights: Option<Vec<u32>>,
    pub named_sets: HashMap<String, NamedSet>,
    pub word_classes: HashMap<String, ResolvedWordClass>,
    pub sentence: Option<SentenceConfig>,
    pub grammar: Option<Vec<String>>,
    pub grammar_weights: Option<Vec<u32>>,
    pub default_stress: Option<StressStrategy>,
    pub default_secondary: bool,
    pub morphology: Option<ResolvedMorphology>,
}

impl Sketch {
    pub fn load(json: &str) -> Result<Resolved, SketchError> {
        let sketch: Sketch = serde_json::from_str(json)?;
        sketch.resolve()
    }

    pub fn resolve(self) -> Result<Resolved, SketchError> {
        if self.word_classes.is_empty() {
            return Err(SketchError::Validation(
                "word_classes must not be empty".into(),
            ));
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

        // Resolve stress configuration.
        let default_strategy = self.stress.as_ref().map(|s| s.default);
        let default_secondary = self.stress.as_ref().is_some_and(|s| s.secondary);

        // Resolve vowel reduction (absent = feature off).
        let reduction = match self.reduction {
            None => ResolvedReduction::default(),
            Some(rc) => {
                // Reduction targets unstressed syllables, so without any stress
                // configuration every syllable would reduce — reject the likely
                // mistake. An explicit "none" (top-level or per-class) is allowed.
                if self.stress.is_none() && self.word_classes.values().all(|wc| wc.stress.is_none())
                {
                    return Err(SketchError::Validation(
                        "reduction requires stress assignment; add a top-level stress section \
                         or per-class stress fields"
                            .into(),
                    ));
                }
                if !(0.0..=1.0).contains(&rc.probability) {
                    return Err(SketchError::Validation(
                        "reduction.probability must be between 0.0 and 1.0".into(),
                    ));
                }
                let parse_target = |target: &str| -> Result<phone::Vowel, SketchError> {
                    let mut chars = target.chars();
                    match (chars.next(), chars.next()) {
                        (Some(c), None) => Ok(phone::Vowel::try_from(c)?),
                        _ => Err(SketchError::Validation(format!(
                            "reduction target must be a single vowel, got {target:?}"
                        ))),
                    }
                };
                let targets = match (rc.target, rc.targets) {
                    (Some(_), Some(_)) => {
                        return Err(SketchError::Validation(
                            "reduction.target and reduction.targets are mutually exclusive".into(),
                        ));
                    }
                    (None, None) => {
                        return Err(SketchError::Validation(
                            "reduction requires either a target or a targets map".into(),
                        ));
                    }
                    (Some(t), None) => ReductionTargets::All(parse_target(&t)?),
                    (None, Some(raw)) => {
                        let mut map = HashMap::new();
                        for (sources, target) in raw {
                            let t = parse_target(&target)?;
                            if sources.is_empty() {
                                return Err(SketchError::Validation(
                                    "reduction.targets keys must not be empty".into(),
                                ));
                            }
                            for ch in sources.chars() {
                                let v = phone::Vowel::try_from(ch)?;
                                if map.insert(v, t).is_some() {
                                    return Err(SketchError::Validation(format!(
                                        "vowel {ch:?} appears in multiple reduction targets"
                                    )));
                                }
                            }
                        }
                        ReductionTargets::Map(map)
                    }
                };
                ResolvedReduction {
                    probability: rc.probability,
                    targets,
                }
            }
        };

        // Resolve word classes.
        let mut resolved_classes = HashMap::new();
        for (name, config) in self.word_classes {
            if config.patterns.is_empty() {
                return Err(SketchError::Validation(format!(
                    "word class {name:?} has no patterns"
                )));
            }
            let (patterns, pattern_weights) = resolve_entries(config.patterns);
            let lexicon = config.lexicon.and_then(|l| l.generate);
            let stress = config.stress.or(default_strategy);
            resolved_classes.insert(
                name,
                ResolvedWordClass {
                    patterns,
                    pattern_weights,
                    lexicon,
                    stress,
                    secondary_stress: default_secondary,
                    reduction: reduction.clone(),
                },
            );
        }

        // Resolve grammar templates (optional).
        let (grammar, grammar_weights) = match self.grammar {
            Some(gr) => {
                let (grammar_strings, gw) = resolve_entries(gr);
                for template in &grammar_strings {
                    for class_name in template.split_ascii_whitespace() {
                        if !resolved_classes.contains_key(class_name) {
                            return Err(SketchError::Validation(format!(
                                "grammar references unknown word class: {class_name:?}"
                            )));
                        }
                    }
                }
                (Some(grammar_strings), gw)
            }
            None => (None, None),
        };

        let morphology = match self.morphology {
            Some(mc) => {
                let decay = mc.decay.unwrap_or(0.5);
                if !(0.0..=1.0).contains(&decay) {
                    return Err(SketchError::Validation(
                        "morphology.decay must be between 0.0 and 1.0".into(),
                    ));
                }
                let resolve_affixes = |affixes: Option<Vec<AffixConfig>>| -> Result<Vec<ResolvedAffix>, SketchError> {
                    let Some(affixes) = affixes else {
                        return Ok(Vec::new());
                    };
                    let mut out = Vec::with_capacity(affixes.len());
                    for affix in affixes {
                        if !(0.0..=1.0).contains(&affix.probability) {
                            return Err(SketchError::Validation(format!(
                                "affix probability must be between 0.0 and 1.0, got {}",
                                affix.probability,
                            )));
                        }
                        if let Some(ref classes) = affix.applies_to {
                            for class_name in classes {
                                if !resolved_classes.contains_key(class_name) {
                                    return Err(SketchError::Validation(format!(
                                        "morphology affix references unknown word class: {class_name:?}",
                                    )));
                                }
                            }
                        }
                        out.push(ResolvedAffix {
                            pattern: affix.pattern,
                            applies_to: affix.applies_to,
                            probability: affix.probability,
                        });
                    }
                    Ok(out)
                };
                let prefixes = resolve_affixes(mc.prefixes)?;
                let suffixes = resolve_affixes(mc.suffixes)?;
                Some(ResolvedMorphology {
                    decay,
                    prefixes,
                    suffixes,
                })
            }
            None => None,
        };

        Ok(Resolved {
            inventory,
            consonant_weights,
            vowel_weights,
            named_sets,
            word_classes: resolved_classes,
            sentence: self.sentence,
            grammar,
            grammar_weights,
            default_stress: default_strategy,
            default_secondary,
            morphology,
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
        PhonemeSet::WithDistribution {
            values,
            distribution,
        } => {
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
        PhonemeSet::WithDistribution {
            values,
            distribution,
        } => {
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

fn resolve_entries(entries: Vec<NamedSetEntry>) -> (Vec<String>, Option<Vec<u32>>) {
    let has_weights = entries
        .iter()
        .any(|e| matches!(e, NamedSetEntry::Weighted { .. }));
    let mut strings = Vec::with_capacity(entries.len());
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
        strings.push(value);
        if let Some(ref mut w) = weights {
            w.push(weight);
        }
    }
    (strings, weights)
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

    /// Helper: minimal valid sketch JSON with a single word class.
    const MINIMAL: &str = r#"{
        "consonants": "ptk",
        "vowels": "aiu",
        "word_classes": { "word": { "patterns": ["CVC"] } }
    }"#;

    #[test]
    fn simple_sketch() {
        let resolved = Sketch::load(MINIMAL).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 3);
        assert_eq!(resolved.inventory.vowels().len(), 3);
        assert_eq!(resolved.consonant_weights.unwrap().len(), 3);
        assert_eq!(resolved.vowel_weights.unwrap().len(), 3);
        assert_eq!(resolved.word_classes["word"].patterns, vec!["CVC"]);
    }

    #[test]
    fn cosine_distribution() {
        let json = r#"{
            "consonants": {
                "values": "pbɣʂʐɟkg",
                "distribution": { "curve": "cosine" }
            },
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let w = resolved.consonant_weights.unwrap();
        assert_eq!(w.len(), 8);
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
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 3);
        assert_eq!(resolved.consonant_weights.unwrap(), vec![30, 20, 10]);
    }

    #[test]
    fn mixed_formats() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": [
                { "value": "a", "weight": 50 },
                { "value": "i", "weight": 30 }
            ],
            "word_classes": { "word": { "patterns": ["CV"] } }
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
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 5);
    }

    #[test]
    fn empty_word_classes_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": {}
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
            "word_classes": { "word": { "patterns": ["CV"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let w = resolved.consonant_weights.unwrap();
        assert_eq!(w.len(), 12);
        let total: u32 = w.iter().sum();
        let first_pct = w[0] as f64 / total as f64;
        let last_pct = w[11] as f64 / total as f64;
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
            "word_classes": { "word": { "patterns": ["CV"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.inventory.consonants().len(), 2);
        assert!(resolved.inventory.consonants()[0].manner() == Some(phone::Manner::Affricate));
    }

    #[test]
    fn named_set_single_segments() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "sets": { "O": ["p", "t", "k"] },
            "word_classes": { "word": { "patterns": ["$OV"] } }
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
            "sets": { "O": ["p", "st", "pl"] },
            "word_classes": { "word": { "patterns": ["$OV"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let onset = &resolved.named_sets["O"];
        assert_eq!(onset.choices.len(), 3);
        assert_eq!(onset.choices[1].len(), 2);
    }

    #[test]
    fn named_set_weighted() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "a",
            "sets": { "O": ["p", {"value": "t", "weight": 5}] },
            "word_classes": { "word": { "patterns": ["$OV"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let onset = &resolved.named_sets["O"];
        assert_eq!(onset.choices.len(), 2);
        assert_eq!(onset.weights.as_ref().unwrap(), &[1, 5]);
    }

    #[test]
    fn named_set_empty_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "a",
            "sets": { "O": [] },
            "word_classes": { "word": { "patterns": ["$OV"] } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn sentence_config_loads() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "sentence": { "words": [3, 8] }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.sentence.unwrap().words, [3, 8]);
    }

    #[test]
    fn sentence_config_absent() {
        let resolved = Sketch::load(MINIMAL).unwrap();
        assert!(resolved.sentence.is_none());
    }

    #[test]
    fn sentence_config_min_zero_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "sentence": { "words": [0, 5] }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn sentence_config_min_exceeds_max_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "sentence": { "words": [8, 3] }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn weighted_class_patterns() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": {
                "word": { "patterns": [
                    {"value": "CVC", "weight": 40},
                    {"value": "CV", "weight": 10}
                ] }
            }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let w = &resolved.word_classes["word"];
        assert_eq!(w.patterns, vec!["CVC", "CV"]);
        assert_eq!(w.pattern_weights, Some(vec![40, 10]));
    }

    #[test]
    fn word_classes_and_grammar() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": {
                "det": { "patterns": ["CV"] },
                "noun": { "patterns": [{"value": "CVC", "weight": 10}, {"value": "CV CVC", "weight": 5}] }
            },
            "grammar": [
                {"value": "det noun", "weight": 20},
                {"value": "noun", "weight": 10}
            ]
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.word_classes.len(), 2);
        let noun = &resolved.word_classes["noun"];
        assert_eq!(noun.patterns, vec!["CVC", "CV CVC"]);
        assert_eq!(noun.pattern_weights, Some(vec![10, 5]));
        let det = &resolved.word_classes["det"];
        assert_eq!(det.patterns, vec!["CV"]);
        assert!(det.pattern_weights.is_none());
        let gr = resolved.grammar.unwrap();
        assert_eq!(gr, vec!["det noun", "noun"]);
        assert_eq!(resolved.grammar_weights, Some(vec![20, 10]));
    }

    #[test]
    fn word_classes_with_lexicon() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": {
                "noun": {
                    "patterns": ["CVC"],
                    "lexicon": { "generate": { "size": 50 } }
                }
            }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(
            resolved.word_classes["noun"].lexicon.as_ref().unwrap().size,
            50
        );
    }

    #[test]
    fn word_classes_without_grammar_ok() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert!(resolved.grammar.is_none());
    }

    #[test]
    fn grammar_references_unknown_class_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "noun": { "patterns": ["CVC"] } },
            "grammar": ["det noun"]
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn word_class_empty_patterns_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "noun": { "patterns": [] } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn stress_config_loads() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "stress": { "default": "trochaic", "secondary": true },
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.default_stress, Some(StressStrategy::Trochaic));
        assert!(resolved.default_secondary);
        assert_eq!(
            resolved.word_classes["word"].stress,
            Some(StressStrategy::Trochaic)
        );
        assert!(resolved.word_classes["word"].secondary_stress);
    }

    #[test]
    fn stress_per_class_override() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "stress": { "default": "trochaic" },
            "word_classes": {
                "noun": { "patterns": ["CVC"] },
                "det": { "patterns": ["CV"], "stress": "none" }
            }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(
            resolved.word_classes["noun"].stress,
            Some(StressStrategy::Trochaic)
        );
        assert_eq!(
            resolved.word_classes["det"].stress,
            Some(StressStrategy::None)
        );
    }

    #[test]
    fn stress_absent_defaults_to_none() {
        let resolved = Sketch::load(MINIMAL).unwrap();
        assert_eq!(resolved.default_stress, None);
        assert!(!resolved.default_secondary);
        assert_eq!(resolved.word_classes["word"].stress, None);
    }

    #[test]
    fn stress_invalid_strategy_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "stress": { "default": "oops" },
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn morphology_loads() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": {
                "noun": { "patterns": ["CVC"] },
                "verb": { "patterns": ["CV"] }
            },
            "morphology": {
                "decay": 0.4,
                "suffixes": [
                    { "pattern": "VC", "applies_to": ["verb"], "probability": 0.3, "label": "past" }
                ],
                "prefixes": [
                    { "pattern": "CV", "probability": 0.1 }
                ]
            }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let morph = resolved.morphology.unwrap();
        assert!((morph.decay - 0.4).abs() < f64::EPSILON);
        assert_eq!(morph.suffixes.len(), 1);
        assert_eq!(morph.suffixes[0].pattern, "VC");
        assert_eq!(
            morph.suffixes[0].applies_to.as_deref().unwrap(),
            &["verb".to_string()]
        );
        assert!((morph.suffixes[0].probability - 0.3).abs() < f64::EPSILON);
        assert_eq!(morph.prefixes.len(), 1);
        assert!(morph.prefixes[0].applies_to.is_none());
    }

    #[test]
    fn morphology_absent_ok() {
        let resolved = Sketch::load(MINIMAL).unwrap();
        assert!(resolved.morphology.is_none());
    }

    #[test]
    fn morphology_default_decay() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "morphology": {
                "suffixes": [{ "pattern": "V", "probability": 0.5 }]
            }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let morph = resolved.morphology.unwrap();
        assert!((morph.decay - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn morphology_invalid_probability_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "morphology": {
                "suffixes": [{ "pattern": "V", "probability": 1.5 }]
            }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn morphology_invalid_decay_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "morphology": { "decay": 2.0 }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_invalid_probability_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 1.5, "target": "ə" }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_resolves_onto_word_classes() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 0.7, "target": "ə" }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let reduction = &resolved.word_classes["word"].reduction;
        assert_eq!(reduction.probability, 0.7);
        assert!(matches!(
            reduction.targets,
            ReductionTargets::All(phone::Vowel::Schwa)
        ));
    }

    #[test]
    fn reduction_without_stress_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "reduction": { "probability": 0.7, "target": "ə" }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_with_only_per_class_stress_ok() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": {
                "word": { "patterns": ["CVC"], "stress": "trochaic" },
                "det":  { "patterns": ["CV"] }
            },
            "reduction": { "probability": 0.7, "target": "ə" }
        }"#;
        assert!(Sketch::load(json).is_ok());
    }

    #[test]
    fn reduction_without_target_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 0.7 }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_target_and_targets_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 0.7, "target": "ə", "targets": { "a": "ə" } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_absent_defaults_to_zero() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        assert_eq!(resolved.word_classes["word"].reduction.probability, 0.0);
    }

    #[test]
    fn reduction_targets_resolve() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aeɛou",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": {
                "probability": 0.8,
                "targets": { "aeɛ": "ə", "o": "u" }
            }
        }"#;
        let resolved = Sketch::load(json).unwrap();
        let ReductionTargets::Map(ref targets) = resolved.word_classes["word"].reduction.targets
        else {
            panic!("expected a targets map");
        };
        assert_eq!(targets.len(), 4);
        assert_eq!(targets[&phone::Vowel::A], phone::Vowel::Schwa);
        assert_eq!(targets[&phone::Vowel::EOpen], phone::Vowel::Schwa);
        assert_eq!(targets[&phone::Vowel::O], phone::Vowel::U);
        assert!(!targets.contains_key(&phone::Vowel::U));
    }

    #[test]
    fn reduction_target_not_single_vowel_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 0.5, "targets": { "a": "əu" } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_target_consonant_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 0.5, "targets": { "t": "ə" } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn reduction_duplicate_source_vowel_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "stress": { "default": "trochaic" },
            "reduction": { "probability": 0.5, "targets": { "ai": "ə", "au": "u" } }
        }"#;
        assert!(Sketch::load(json).is_err());
    }

    #[test]
    fn morphology_unknown_class_error() {
        let json = r#"{
            "consonants": "ptk",
            "vowels": "aiu",
            "word_classes": { "word": { "patterns": ["CVC"] } },
            "morphology": {
                "suffixes": [{ "pattern": "V", "applies_to": ["nope"], "probability": 0.5 }]
            }
        }"#;
        assert!(Sketch::load(json).is_err());
    }
}
