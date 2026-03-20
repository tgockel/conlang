//! # Generate
//!
//! This module concerns generation of words from rule sets.

use crate::phone;
use rand::Rng;
use smallvec::SmallVec;
use std::fmt;
use thiserror::Error;

/// Optional per-category weights to use during generation.
/// When a field is `None`, uniform distribution is used for that category.
pub struct InventoryWeights {
    pub consonant_weights: Option<Vec<u32>>,
    pub vowel_weights: Option<Vec<u32>>,
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

    pub fn parse(
        src: &str,
        inventory: &phone::Inventory,
        weights: Option<&InventoryWeights>,
    ) -> Result<Self, ParseError> {
        let mut syllables = SmallVec::new();

        for syl_src in src.split_ascii_whitespace() {
            syllables.push(SyllableGenerator::parse(syl_src, inventory, weights)?);
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
            if let Some(seg) = ph.generate(rng) {
                out.push(seg);
            }
        }

        phone::Syllable::new(out.as_slice())
    }

    pub(super) fn parse(
        src: &str,
        inventory: &phone::Inventory,
        weights: Option<&InventoryWeights>,
    ) -> Result<Self, ParseError> {
        let mut phonemes = SmallVec::new();
        let mut rem = src;
        while !rem.is_empty() {
            let (phoneme, leftover) = SegmentGenerator::parse(rem, inventory, weights)?;
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
    choices: SmallVec<[phone::Segment; 8]>,
    weights: SmallVec<[u32; 8]>,
    optional: bool,
}

impl SegmentGenerator {
    pub(super) fn parse<'a>(
        src: &'a str,
        inventory: &phone::Inventory,
        weights: Option<&InventoryWeights>,
    ) -> Result<(Self, &'a str), ParseError> {
        let Some(first) = src.chars().nth(0) else {
            return Err(ParseError::NoInput);
        };

        let c_weights = weights.and_then(|w| w.consonant_weights.as_deref());
        let v_weights = weights.and_then(|w| w.vowel_weights.as_deref());

        match first {
            'C' => Ok(Self::from_segments(src, inventory.consonants(), c_weights)),
            'V' => Ok(Self::from_segments(src, inventory.vowels(), v_weights)),
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
                    choices.push(seg);
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

                let (mut inner_gen, leftover) = Self::parse(inner, inventory, weights)?;
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
            _ => {
                if let Ok(places) = phone::Place::try_from(first) {
                    Ok(Self::from_segments_filtered(
                        src,
                        inventory.consonants(),
                        c_weights,
                        |x| x.place().is_some_and(|p| places.contains(&p)),
                    ))
                } else if let Ok(manners) = phone::Manner::try_from(first) {
                    Ok(Self::from_segments_filtered(
                        src,
                        inventory.consonants(),
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
            choices: options.iter().copied().collect(),
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
                choices.push(*seg);
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

    pub fn generate(&self, rng: &mut impl Rng) -> Option<phone::Segment> {
        if self.optional && rng.next_u64() % 2 == 0 {
            return None;
        }
        if self.weights.is_empty() {
            Some(self.choices[rng.next_u64() as usize % self.choices.len()])
        } else {
            let total: u64 = self.weights.iter().map(|&w| w as u64).sum();
            let mut roll = rng.next_u64() % total;
            for (seg, &w) in self.choices.iter().zip(self.weights.iter()) {
                if roll < w as u64 {
                    return Some(*seg);
                }
                roll -= w as u64;
            }
            Some(*self.choices.last().unwrap())
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

    #[test]
    fn parsing() {
        let inventory = phone::Inventory::with_everything();
        let inputs = &["C", "V", "CV", "VVC"];
        for input in inputs.iter() {
            WordGenerator::parse(input, &inventory, None).unwrap();
        }
    }

    #[test]
    fn parse_bracket_basic() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("[ptk]VC", &inventory, None).unwrap();
        assert_eq!(format!("{wg}"), "[ptk]VC");
    }

    #[test]
    fn parse_bracket_vowels() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("C[aiu]C", &inventory, None).unwrap();
        assert_eq!(format!("{wg}"), "C[aiu]C");
    }

    #[test]
    fn parse_bracket_empty_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("[]V", &inventory, None),
            Err(ParseError::EmptyBracket)
        ));
    }

    #[test]
    fn parse_bracket_unclosed_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("[ptk", &inventory, None),
            Err(ParseError::UnclosedBracket)
        ));
    }

    #[test]
    fn parse_bracket_unknown_char_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("[pWk]", &inventory, None),
            Err(ParseError::InvalidPhoneme(_))
        ));
    }

    #[test]
    fn parse_optional_consonant() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("(C)VC", &inventory, None).unwrap();
        assert_eq!(format!("{wg}"), "(C)VC");
    }

    #[test]
    fn parse_optional_bracket() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("([ptk])VC", &inventory, None).unwrap();
        assert_eq!(format!("{wg}"), "([ptk])VC");
    }

    #[test]
    fn parse_optional_empty_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("()VC", &inventory, None),
            Err(ParseError::EmptyParen)
        ));
    }

    #[test]
    fn parse_optional_unclosed_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(matches!(
            WordGenerator::parse("(CVC", &inventory, None),
            Err(ParseError::UnclosedParen)
        ));
    }

    #[test]
    fn parse_optional_multiple_segments_error() {
        let inventory = phone::Inventory::with_everything();
        assert!(WordGenerator::parse("(CV)", &inventory, None).is_err());
    }

    #[test]
    fn generate_bracket_produces_valid_segment() {
        let inventory = phone::Inventory::with_everything();
        let wg = WordGenerator::parse("[ptk]V", &inventory, None).unwrap();
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
        let wg = WordGenerator::parse("(C)V", &inventory, None).unwrap();
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
}
