//! # Generate
//!
//! This module concerns generation of words from rule sets.

use crate::phone;
use crate::sketch;
use rand::Rng;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Optional per-category weights to use during generation.
/// When a field is `None`, uniform distribution is used for that category.
pub struct InventoryWeights {
    pub consonant_weights: Option<Vec<u32>>,
    pub vowel_weights: Option<Vec<u32>>,
}

/// Everything needed to parse a pattern string into generators.
pub struct ParseContext<'a> {
    pub inventory: &'a phone::Inventory,
    pub weights: Option<&'a InventoryWeights>,
    pub named_sets: &'a HashMap<String, sketch::NamedSet>,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("no input")]
    NoInput,
    #[error("unrecognized character: '{0}'")]
    UnknownCharacter(char),
    #[error("unclosed bracket '[' without matching ']'")]
    UnclosedBracket,
    #[error("empty bracket set '[]'")]
    EmptyBracket,
    #[error("unclosed parenthesis '(' without matching ')'")]
    UnclosedParen,
    #[error("empty parentheses '()'")]
    EmptyParen,
    #[error("invalid phoneme in bracket set: {0}")]
    InvalidPhoneme(#[from] phone::ParseError),
    #[error("'$' without a set name")]
    EmptySetName,
    #[error("unclosed '${{' without matching '}}'")]
    UnclosedSetName,
    #[error("unknown named set: '{0}'")]
    UnknownSet(String),
}

#[derive(Clone, PartialEq)]
pub struct WordGenerator {
    syllables: SmallVec<[SyllableGenerator; 4]>,
}

impl WordGenerator {
    pub fn generate(&self, rng: &mut impl Rng) -> SmallVec<[phone::Syllable; 4]> {
        let mut out = SmallVec::new();
        for syl in self.syllables.iter() {
            out.push(syl.generate(rng));
        }
        out
    }

    pub fn parse(src: &str, ctx: &ParseContext) -> Result<Self, ParseError> {
        let mut syllables = SmallVec::new();

        for syl_src in src.split_ascii_whitespace() {
            syllables.push(SyllableGenerator::parse(syl_src, ctx)?);
        }

        if syllables.is_empty() {
            Err(ParseError::NoInput)
        } else {
            Ok(Self { syllables })
        }
    }
}

impl fmt::Display for WordGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for syl in self.syllables.iter() {
            if first {
                first = false;
            } else {
                f.write_str(" ")?;
            }
            write!(f, "{syl}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for WordGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WordGenerator({self})")
    }
}

#[derive(Clone, PartialEq)]
pub struct SyllableGenerator {
    phonemes: SmallVec<[SegmentGenerator; 4]>,
}

impl SyllableGenerator {
    pub fn generate(&self, rng: &mut impl Rng) -> phone::Syllable {
        let mut out = SmallVec::<[phone::Segment; 4]>::with_capacity(self.phonemes.len());
        for ph in self.phonemes.iter() {
            if let Some(segs) = ph.generate(rng) {
                out.extend(segs);
            }
        }

        phone::Syllable::new(out.as_slice())
    }

    pub(super) fn parse(src: &str, ctx: &ParseContext) -> Result<Self, ParseError> {
        let mut phonemes = SmallVec::new();
        let mut rem = src;
        while !rem.is_empty() {
            let (phoneme, leftover) = SegmentGenerator::parse(rem, ctx)?;
            phonemes.push(phoneme);
            rem = leftover;
        }

        if phonemes.is_empty() {
            Err(ParseError::NoInput)
        } else {
            Ok(Self { phonemes })
        }
    }
}

impl fmt::Display for SyllableGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for phoneme in self.phonemes.iter() {
            write!(f, "{phoneme}")?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct SegmentGenerator {
    display: String,
    choices: SmallVec<[SmallVec<[phone::Segment; 2]>; 8]>,
    weights: SmallVec<[u32; 8]>,
    optional: bool,
}

impl SegmentGenerator {
    pub(super) fn parse<'a>(
        src: &'a str,
        ctx: &ParseContext,
    ) -> Result<(Self, &'a str), ParseError> {
        let Some(first) = src.chars().nth(0) else {
            return Err(ParseError::NoInput);
        };

        let c_weights = ctx.weights.and_then(|w| w.consonant_weights.as_deref());
        let v_weights = ctx.weights.and_then(|w| w.vowel_weights.as_deref());

        match first {
            'C' => Ok(Self::from_segments(src, ctx.inventory.consonants(), c_weights)),
            'V' => Ok(Self::from_segments(src, ctx.inventory.vowels(), v_weights)),
            '[' => {
                let close = src.find(']').ok_or(ParseError::UnclosedBracket)?;
                let inner = &src[1..close];
                if inner.is_empty() {
                    return Err(ParseError::EmptyBracket);
                }

                let mut choices = SmallVec::new();
                let mut remaining_inner = inner;
                while !remaining_inner.is_empty() {
                    let (seg, rest) = phone::Segment::parse_ipa(remaining_inner)?;
                    choices.push(smallvec![seg]);
                    remaining_inner = rest;
                }

                let display = src[..=close].to_string();
                let remainder = &src[close + 1..];
                Ok((
                    Self {
                        display,
                        choices,
                        weights: SmallVec::new(),
                        optional: false,
                    },
                    remainder,
                ))
            }
            '(' => {
                let close = src.find(')').ok_or(ParseError::UnclosedParen)?;
                let inner = &src[1..close];
                if inner.is_empty() {
                    return Err(ParseError::EmptyParen);
                }

                let (mut inner_gen, leftover) = Self::parse(inner, ctx)?;
                if !leftover.is_empty() {
                    return Err(ParseError::UnknownCharacter(
                        leftover.chars().next().unwrap(),
                    ));
                }

                inner_gen.optional = true;
                inner_gen.display = src[..=close].to_string();

                let remainder = &src[close + 1..];
                Ok((inner_gen, remainder))
            }
            '$' => {
                let (name, name_end) = if src.as_bytes().get(1) == Some(&b'{') {
                    // ${name} syntax for multi-character names
                    let close = src.find('}').ok_or(ParseError::UnclosedSetName)?;
                    let name = &src[2..close];
                    if name.is_empty() {
                        return Err(ParseError::EmptySetName);
                    }
                    if !ctx.named_sets.contains_key(name) {
                        return Err(ParseError::UnknownSet(name.to_string()));
                    }
                    (name, close + 1) // consume past '}'
                } else {
                    // $X shorthand: longest-prefix-match against known set names
                    let name_start = 1; // skip '$', which is ASCII
                    let max_end = src[name_start..]
                        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .map(|pos| name_start + pos)
                        .unwrap_or(src.len());
                    if name_start == max_end {
                        return Err(ParseError::EmptySetName);
                    }
                    let full = &src[name_start..max_end];
                    let mut found = None;
                    for end in (1..=full.len()).rev() {
                        let candidate = &full[..end];
                        if ctx.named_sets.contains_key(candidate) {
                            found = Some((candidate, name_start + end));
                            break;
                        }
                    }
                    found.ok_or_else(|| ParseError::UnknownSet(full.to_string()))?
                };
                let set = &ctx.named_sets[name];
                let display = src[..name_end].to_string();
                let remainder = &src[name_end..];
                Ok((
                    Self {
                        display,
                        choices: set.choices.iter().cloned().collect(),
                        weights: set
                            .weights
                            .as_ref()
                            .map(|w| w.iter().copied().collect())
                            .unwrap_or_default(),
                        optional: false,
                    },
                    remainder,
                ))
            }
            _ => {
                if let Ok(places) = phone::Place::try_from(first) {
                    Ok(Self::from_segments_filtered(
                        src,
                        ctx.inventory.consonants(),
                        c_weights,
                        |x| x.place().is_some_and(|p| places.contains(&p)),
                    ))
                } else if let Ok(manners) = phone::Manner::try_from(first) {
                    Ok(Self::from_segments_filtered(
                        src,
                        ctx.inventory.consonants(),
                        c_weights,
                        |x| x.manner().is_some_and(|m| manners.contains(&m)),
                    ))
                } else {
                    Err(ParseError::UnknownCharacter(first))
                }
            }
        }
    }

    fn from_segments<'a>(
        src: &'a str,
        options: &[phone::Segment],
        weights: Option<&[u32]>,
    ) -> (Self, &'a str) {
        let first_len = src.chars().next().unwrap().len_utf8();
        let out = Self {
            display: src[..first_len].into(),
            choices: options.iter().map(|&s| smallvec![s]).collect(),
            weights: weights
                .map(|w| w.iter().copied().collect())
                .unwrap_or_default(),
            optional: false,
        };
        (out, &src[first_len..])
    }

    fn from_segments_filtered<'a>(
        src: &'a str,
        options: &[phone::Segment],
        weights: Option<&[u32]>,
        filter: impl Fn(&phone::Segment) -> bool,
    ) -> (Self, &'a str) {
        let first_len = src.chars().next().unwrap().len_utf8();
        let mut choices = SmallVec::new();
        let mut filtered_weights = SmallVec::new();
        for (i, seg) in options.iter().enumerate() {
            if filter(seg) {
                choices.push(smallvec![*seg]);
                if let Some(w) = weights.and_then(|ws| ws.get(i)) {
                    filtered_weights.push(*w);
                }
            }
        }
        if weights.is_none() {
            filtered_weights = SmallVec::new();
        }
        let out = Self {
            display: src[..first_len].into(),
            choices,
            weights: filtered_weights,
            optional: false,
        };
        (out, &src[first_len..])
    }

    pub fn generate(&self, rng: &mut impl Rng) -> Option<SmallVec<[phone::Segment; 2]>> {
        if self.optional && rng.next_u64() % 2 == 0 {
            return None;
        }
        if self.weights.is_empty() {
            let chosen = &self.choices[rng.next_u64() as usize % self.choices.len()];
            Some(chosen.clone())
        } else {
            let total: u64 = self.weights.iter().map(|&w| w as u64).sum();
            let mut roll = rng.next_u64() % total;
            for (chosen, &w) in self.choices.iter().zip(self.weights.iter()) {
                if roll < w as u64 {
                    return Some(chosen.clone());
                }
                roll -= w as u64;
            }
            Some(self.choices.last().unwrap().clone())
        }
    }
}

impl PartialEq for SegmentGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.choices == other.choices
            && self.weights == other.weights
            && self.optional == other.optional
    }
}

impl fmt::Display for SegmentGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.display)
    }
}

impl fmt::Debug for SegmentGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Phoneme({self})")
    }
}

#[cfg(test)]
mod gen_tests {
    use super::*;
    use crate::phone;

    fn ctx(inventory: &phone::Inventory) -> ParseContext<'_> {
        static EMPTY: std::sync::LazyLock<HashMap<String, sketch::NamedSet>> =
            std::sync::LazyLock::new(HashMap::new);
        ParseContext {
            inventory,
            weights: None,
            named_sets: &EMPTY,
        }
    }

    #[test]
    fn parsing() {
        let inventory = phone::Inventory::with_everything();
        let inputs = &["C", "V", "CV", "VVC"];
        for input in inputs.iter() {
            WordGenerator::parse(input, &ctx(&inventory)).unwrap();
        }
    }

    #[test]
    fn parse_bracket_basic() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("[ptk]VC", &ctx(&inventory)).unwrap();
        assert_eq!(format!("{wg}"), "[ptk]VC");
    }

    #[test]
    fn parse_bracket_vowels() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("C[aiu]C", &ctx(&inventory)).unwrap();
        assert_eq!(format!("{wg}"), "C[aiu]C");
    }

    #[test]
    fn parse_bracket_empty_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("[]V", &ctx(&inventory)),
            Err(ParseError::EmptyBracket)
        ));
    }

    #[test]
    fn parse_bracket_unclosed_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("[ptk", &ctx(&inventory)),
            Err(ParseError::UnclosedBracket)
        ));
    }

    #[test]
    fn parse_bracket_unknown_char_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("[pWk]", &ctx(&inventory)),
            Err(ParseError::InvalidPhoneme(_))
        ));
    }

    #[test]
    fn parse_optional_consonant() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("(C)VC", &ctx(&inventory)).unwrap();
        assert_eq!(format!("{wg}"), "(C)VC");
    }

    #[test]
    fn parse_optional_bracket() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("([ptk])VC", &ctx(&inventory)).unwrap();
        assert_eq!(format!("{wg}"), "([ptk])VC");
    }

    #[test]
    fn parse_optional_empty_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("()VC", &ctx(&inventory)),
            Err(ParseError::EmptyParen)
        ));
    }

    #[test]
    fn parse_optional_unclosed_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("(CVC", &ctx(&inventory)),
            Err(ParseError::UnclosedParen)
        ));
    }

    #[test]
    fn parse_optional_multiple_segments_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(WordGenerator::parse("(CV)", &ctx(&inventory)).is_err());
    }

    #[test]
    fn generate_bracket_produces_valid_segment() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("[ptk]V", &ctx(&inventory)).unwrap();
        let mut rng = rand::rng();
        let valid = [
            phone::Segment::from(phone::Consonant::P),
            phone::Segment::from(phone::Consonant::T),
            phone::Segment::from(phone::Consonant::K),
        ];
        for _ in 0..50 {
            let word = wg.generate(&mut rng);
            assert_eq!(word.len(), 1);
            let first = word[0].segments()[0];
            assert!(valid.contains(&first), "unexpected segment: {first}");
        }
    }

    #[test]
    fn generate_optional_sometimes_absent() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("(C)V", &ctx(&inventory)).unwrap();
        let mut rng = rand::rng();
        let mut had_one = false;
        let mut had_two = false;
        for _ in 0..200 {
            let word = wg.generate(&mut rng);
            let len = word[0].segments().len();
            match len {
                1 => had_one = true,
                2 => had_two = true,
                _ => panic!("unexpected segment count: {len}"),
            }
        }
        assert!(had_one, "optional segment was never absent");
        assert!(had_two, "optional segment was never present");
    }

    #[test]
    fn parse_named_set_single_segments() {
        let inventory = phone::Inventory::with_everything();
        let mut sets = HashMap::new();
        sets.insert(
            "O".to_string(),
            sketch::NamedSet {
                choices: vec![
                    smallvec![phone::Segment::from(phone::Consonant::P)],
                    smallvec![phone::Segment::from(phone::Consonant::T)],
                    smallvec![phone::Segment::from(phone::Consonant::K)],
                ],
                weights: None,
            },
        );
        let ctx = ParseContext {
            inventory: &inventory,
            weights: None,
            named_sets: &sets,
        };
        let wg = WordGenerator::parse("$OV", &ctx).unwrap();
        assert_eq!(format!("{wg}"), "$OV");
    }

    #[test]
    fn parse_named_set_with_clusters() {
        let inventory = phone::Inventory::with_everything();
        let mut sets = HashMap::new();
        sets.insert(
            "O".to_string(),
            sketch::NamedSet {
                choices: vec![
                    smallvec![phone::Segment::from(phone::Consonant::P)],
                    smallvec![
                        phone::Segment::from(phone::Consonant::S),
                        phone::Segment::from(phone::Consonant::T),
                    ],
                ],
                weights: None,
            },
        );
        sets.insert(
            "K".to_string(),
            sketch::NamedSet {
                choices: vec![
                    smallvec![phone::Segment::from(phone::Consonant::K)],
                    smallvec![
                        phone::Segment::from(phone::Consonant::N),
                        phone::Segment::from(phone::Consonant::D),
                    ],
                ],
                weights: None,
            },
        );
        let ctx = ParseContext {
            inventory: &inventory,
            weights: None,
            named_sets: &sets,
        };
        let wg = WordGenerator::parse("$OV$K", &ctx).unwrap();
        let mut rng = rand::rng();
        for _ in 0..50 {
            let word = wg.generate(&mut rng);
            assert_eq!(word.len(), 1);
            // Syllable should have 3-5 segments depending on cluster selection
            let seg_count = word[0].segments().len();
            assert!(
                (3..=5).contains(&seg_count),
                "unexpected segment count: {seg_count}"
            );
        }
    }

    #[test]
    fn parse_optional_named_set() {
        let inventory = phone::Inventory::with_everything();
        let mut sets = HashMap::new();
        sets.insert(
            "O".to_string(),
            sketch::NamedSet {
                choices: vec![
                    smallvec![phone::Segment::from(phone::Consonant::P)],
                    smallvec![phone::Segment::from(phone::Consonant::T)],
                ],
                weights: None,
            },
        );
        let ctx = ParseContext {
            inventory: &inventory,
            weights: None,
            named_sets: &sets,
        };
        let wg = WordGenerator::parse("($O)V", &ctx).unwrap();
        assert_eq!(format!("{wg}"), "($O)V");
        let mut rng = rand::rng();
        let mut had_one = false;
        let mut had_two = false;
        for _ in 0..200 {
            let word = wg.generate(&mut rng);
            let len = word[0].segments().len();
            match len {
                1 => had_one = true,
                2 => had_two = true,
                _ => panic!("unexpected segment count: {len}"),
            }
        }
        assert!(had_one, "optional named set was never absent");
        assert!(had_two, "optional named set was never present");
    }

    #[test]
    fn parse_unknown_set_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("$fooV", &ctx(&inventory)),
            Err(ParseError::UnknownSet(_))
        ));
    }

    #[test]
    fn parse_empty_set_name_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("$", &ctx(&inventory)),
            Err(ParseError::EmptySetName)
        ));
    }

    #[test]
    fn parse_braced_named_set() {
        let inventory = phone::Inventory::with_everything();
        let mut sets = HashMap::new();
        sets.insert(
            "onset".to_string(),
            sketch::NamedSet {
                choices: vec![smallvec![phone::Segment::from(phone::Consonant::P)]],
                weights: None,
            },
        );
        sets.insert(
            "coda".to_string(),
            sketch::NamedSet {
                choices: vec![smallvec![phone::Segment::from(phone::Consonant::K)]],
                weights: None,
            },
        );
        let ctx = ParseContext {
            inventory: &inventory,
            weights: None,
            named_sets: &sets,
        };
        let wg = WordGenerator::parse("${onset}V${coda}", &ctx).unwrap();
        assert_eq!(format!("{wg}"), "${onset}V${coda}");
    }

    #[test]
    fn parse_optional_braced_set() {
        let inventory = phone::Inventory::with_everything();
        let mut sets = HashMap::new();
        sets.insert(
            "onset".to_string(),
            sketch::NamedSet {
                choices: vec![smallvec![phone::Segment::from(phone::Consonant::P)]],
                weights: None,
            },
        );
        let ctx = ParseContext {
            inventory: &inventory,
            weights: None,
            named_sets: &sets,
        };
        let wg = WordGenerator::parse("(${onset})V", &ctx).unwrap();
        let s = wg.to_string();
        assert!(s.contains("onset"), "display should contain set name: {s}");
    }

    #[test]
    fn parse_empty_braced_set_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("${}V", &ctx(&inventory)),
            Err(ParseError::EmptySetName)
        ));
    }

    #[test]
    fn parse_unclosed_braced_set_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("${foo", &ctx(&inventory)),
            Err(ParseError::UnclosedSetName)
        ));
    }

    #[test]
    fn parse_unknown_braced_set_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("${nope}V", &ctx(&inventory)),
            Err(ParseError::UnknownSet(_))
        ));
    }
}
