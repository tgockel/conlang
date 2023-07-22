use crate::phone;
use rand::Rng;
use smallvec::SmallVec;
use std::{fmt, str::FromStr};
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
    phonemes: SmallVec<[PhonemeGenerator; 4]>,
}

impl SyllableGenerator {
    pub fn generate(&self, rng: &mut impl Rng) -> phone::Syllable {
        let mut out = SmallVec::<[phone::Phoneme; 4]>::with_capacity(self.phonemes.len());
        for ph in self.phonemes.iter() {
            out.push(ph.generate(rng));
        }

        phone::Syllable::new(out.as_slice())
    }

    pub(super) fn parse(src: &str, inventory: &phone::Inventory) -> Result<Self, ParseError> {
        let mut phonemes = SmallVec::new();
        let mut rem = src;
        while !rem.is_empty() {
            let (phoneme, leftover) = PhonemeGenerator::parse(rem, inventory)?;
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
pub struct PhonemeGenerator {
    display: String,
    choices: SmallVec<[phone::Phoneme; 8]>,
    weights: SmallVec<[u8; 8]>,
}

impl PhonemeGenerator {
    pub(super) fn parse<'a>(
        src: &'a str,
        inventory: &phone::Inventory,
    ) -> Result<(Self, &'a str), ParseError> {
        let Some(first) = src.chars().nth(0) else {
            return Err(ParseError::NoInput)
        };

        match first {
            'C' => Ok(Self::from_character_class(src, inventory.consonants())),
            'V' => Ok(Self::from_character_class(src, inventory.vowels())),
            '[' => todo!(),
            '(' => todo!(),
            _ => {
                if let Ok(place) = phone::Place::try_from(first) {
                    Ok(Self::from_character_class_filtered(
                        src,
                        inventory.consonants(),
                        |x| x.place() == place,
                    ))
                } else if let Ok(manner) = phone::Manner::try_from(first) {
                    Ok(Self::from_character_class_filtered(
                        src,
                        inventory.consonants(),
                        |x| x.manner() == manner,
                    ))
                } else {
                    todo!()
                }
            }
        }
    }

    fn from_character_class<'a, T: Into<phone::Phoneme> + Copy>(
        src: &'a str,
        options: &[T],
    ) -> (Self, &'a str) {
        let out = Self {
            display: src[..1].into(),
            choices: options.iter().map(|x| (*x).into()).collect(),
            weights: SmallVec::new(),
        };
        (out, &src[1..])
    }

    fn from_character_class_filtered<'a, T: Into<phone::Phoneme> + Copy>(
        src: &'a str,
        options: &[T],
        filter: impl Fn(&T) -> bool,
    ) -> (Self, &'a str) {
        let out = Self {
            display: src[..1].into(),
            choices: options
                .iter()
                .filter_map(|x| if filter(x) { Some((*x).into()) } else { None })
                .collect(),
            weights: SmallVec::new(),
        };
        (out, &src[1..])
    }

    pub fn generate(&self, rng: &mut impl Rng) -> phone::Phoneme {
        self.choices[rng.next_u64() as usize % self.choices.len()]
    }
}

impl PartialEq for PhonemeGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.choices == other.choices && self.weights == other.weights
    }
}

impl fmt::Display for PhonemeGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.display)
    }
}

impl fmt::Debug for PhonemeGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Phoneme({self})")
    }
}

#[cfg(test)]
mod gen_tests {
    use super::*;

    #[test]
    fn parsing() {
        let inputs = &["C", "V", "CV", "VVC"];
        for input in inputs.iter() {
            WordGenerator::from_str(input).unwrap();
        }
    }
}
