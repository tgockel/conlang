//! # Generate
//!
//! This module concerns generation of words from rule sets.

use crate::phone;
use rand::Rng;
use smallvec::SmallVec;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("no input")]
    NoInput,
    #[error("unrecognized character: '{0}'")]
    UnknownCharacter(char),
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

    pub fn parse(src: &str, inventory: &phone::Inventory) -> Result<Self, ParseError> {
        let mut syllables = SmallVec::new();

        for syl_src in src.split_ascii_whitespace() {
            syllables.push(SyllableGenerator::parse(syl_src, inventory)?);
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
            out.push(ph.generate(rng));
        }

        phone::Syllable::new(out.as_slice())
    }

    pub(super) fn parse(src: &str, inventory: &phone::Inventory) -> Result<Self, ParseError> {
        let mut phonemes = SmallVec::new();
        let mut rem = src;
        while !rem.is_empty() {
            let (phoneme, leftover) = SegmentGenerator::parse(rem, inventory)?;
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
    weights: SmallVec<[u8; 8]>,
}

impl SegmentGenerator {
    pub(super) fn parse<'a>(
        src: &'a str,
        inventory: &phone::Inventory,
    ) -> Result<(Self, &'a str), ParseError> {
        let Some(first) = src.chars().nth(0) else {
            return Err(ParseError::NoInput);
        };

        match first {
            'C' => Ok(Self::from_segments(src, inventory.consonants())),
            'V' => Ok(Self::from_segments(src, inventory.vowels())),
            '[' => todo!(),
            '(' => todo!(),
            _ => {
                if let Ok(places) = phone::Place::try_from(first) {
                    Ok(Self::from_segments_filtered(
                        src,
                        inventory.consonants(),
                        |x| x.place().is_some_and(|p| places.contains(&p)),
                    ))
                } else if let Ok(manners) = phone::Manner::try_from(first) {
                    Ok(Self::from_segments_filtered(
                        src,
                        inventory.consonants(),
                        |x| x.manner().is_some_and(|m| manners.contains(&m)),
                    ))
                } else {
                    todo!()
                }
            }
        }
    }

    fn from_segments<'a>(src: &'a str, options: &[phone::Segment]) -> (Self, &'a str) {
        let out = Self {
            display: src[..1].into(),
            choices: options.iter().copied().collect(),
            weights: SmallVec::new(),
        };
        (out, &src[1..])
    }

    fn from_segments_filtered<'a>(
        src: &'a str,
        options: &[phone::Segment],
        filter: impl Fn(&phone::Segment) -> bool,
    ) -> (Self, &'a str) {
        let out = Self {
            display: src[..1].into(),
            choices: options.iter().filter(|x| filter(x)).copied().collect(),
            weights: SmallVec::new(),
        };
        (out, &src[1..])
    }

    pub fn generate(&self, rng: &mut impl Rng) -> phone::Segment {
        self.choices[rng.next_u64() as usize % self.choices.len()]
    }
}

impl PartialEq for SegmentGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.choices == other.choices && self.weights == other.weights
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
            WordGenerator::parse(input, &inventory).unwrap();
        }
    }
}
