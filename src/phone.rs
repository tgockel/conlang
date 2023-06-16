//! # Phonemes
//!
//! Phonetics form the basis of spoken language.

use std::{
    error::Error,
    fmt::{self, Write},
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    NoInput,
    TooManyCharacters,
    UnknownCharacter(char),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoInput => write!(f, "no input"),
            Self::TooManyCharacters => write!(f, "too many characters in input"),
            Self::UnknownCharacter(c) => write!(f, "unknown character '{c}'"),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for ParseError {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Consonant {
    // == Plosive ==
    P,           // <C- p
    B,           // <C- b
    T,           // <C- t
    D,           // <C- d
    TRetroflex,  // <C- ʈ
    DRetroflex,  // <C- ɖ
    C,           // <C- c
    JPalatal,    // <C- ɟ
    K,           // <C- k
    G,           // <C- g
    Q,           // <C- q
    GUvular,     // <C- ɢ
    GlottalStop, // <C- ʔ
    // == Nasal ==
    MBiLabial,    // <C- m
    MLabioDental, // <C- ɱ
    N,            // <C- n
    NRetroflex,   // <C- ɳ
    NPalatal,     // <C- ɲ
    NVelar,       // <C- ŋ
    NUvular,      // <C- ɴ
    // == Trill ==
    Brr,  // <C- ʙ
    Rrr,  // <C- r
    Rhrr, // <C- ʀ
    // == Tap or Flap ==
    VTap,  // <C- ⱱ
    RTap,  // <C- ɾ
    RTap2, // <C- ɽ
    // == Fricative ==
    Phi,    // <C- ɸ
    Beta,   // <C- β
    F,      // <C- f
    V,      // <C- v
    Theta,  // <C- θ
    Del,    // <C- ð
    S,      // <C- s
    Z,      // <C- z
    Shh,    // <C- ʃ
    Zhh,    // <C- ʒ
    Sh,     // <C- ʂ
    Zh,     // <C- ʐ
    Ch,     // <C- ç
    JCurl,  // <C- ʝ
    X,      // <C- x
    Y,      // <C- ɣ
    Xh,     // <C- χ
    Yr,     // <C- ʁ
    HhLong, // <C- ħ
    Crook,  // <C- ʕ
    H,      // <C- h
    Hh,     // <C- ɦ
    // == Lateral Fricative ==
    LBelt, // <C- ɬ
    Lezh,  // <C- ɮ
    // == Approximant ==
    VHook,     // <C- ʋ
    RTilt,     // <C- ɹ
    RTiltHook, // <C- ɻ
    J,         // <C- j
    MTiltTail, // <C- ɰ
    // == Lateral Approximant ==
    L,      // <C- l
    Ll,     // <C- ɭ
    Lambda, // <C- ʎ
    LCap,   // <C- ʟ
}

const ALL_CONSONANTS: [Consonant; 59] = [
    Consonant::P,
    Consonant::B,
    Consonant::T,
    Consonant::D,
    Consonant::TRetroflex,
    Consonant::DRetroflex,
    Consonant::C,
    Consonant::JPalatal,
    Consonant::K,
    Consonant::G,
    Consonant::Q,
    Consonant::GUvular,
    Consonant::GlottalStop,
    Consonant::MBiLabial,
    Consonant::MLabioDental,
    Consonant::N,
    Consonant::NRetroflex,
    Consonant::NPalatal,
    Consonant::NVelar,
    Consonant::NUvular,
    Consonant::Brr,
    Consonant::Rrr,
    Consonant::Rhrr,
    Consonant::VTap,
    Consonant::RTap,
    Consonant::RTap2,
    Consonant::Phi,
    Consonant::Beta,
    Consonant::F,
    Consonant::V,
    Consonant::Theta,
    Consonant::Del,
    Consonant::S,
    Consonant::Z,
    Consonant::Shh,
    Consonant::Zhh,
    Consonant::Sh,
    Consonant::Zh,
    Consonant::Ch,
    Consonant::JCurl,
    Consonant::X,
    Consonant::Y,
    Consonant::Xh,
    Consonant::Yr,
    Consonant::HhLong,
    Consonant::Crook,
    Consonant::H,
    Consonant::Hh,
    Consonant::LBelt,
    Consonant::Lezh,
    Consonant::VHook,
    Consonant::RTilt,
    Consonant::RTiltHook,
    Consonant::J,
    Consonant::MTiltTail,
    Consonant::L,
    Consonant::Ll,
    Consonant::Lambda,
    Consonant::LCap,
];

impl Consonant {
    pub fn code(&self) -> &'static str {
        match self {
            Self::P => "p",
            Self::B => "b",
            Self::T => "t",
            Self::D => "d",
            Self::TRetroflex => "ʈ",
            Self::DRetroflex => "ɖ",
            Self::C => "c",
            Self::JPalatal => "ɟ",
            Self::K => "k",
            Self::G => "g",
            Self::Q => "q",
            Self::GUvular => "ɢ",
            Self::GlottalStop => "ʔ",
            Self::MBiLabial => "m",
            Self::MLabioDental => "ɱ",
            Self::N => "n",
            Self::NRetroflex => "ɳ",
            Self::NPalatal => "ɲ",
            Self::NVelar => "ŋ",
            Self::NUvular => "ɴ",
            Self::Brr => "ʙ",
            Self::Rrr => "r",
            Self::Rhrr => "ʀ",
            Self::VTap => "ⱱ",
            Self::RTap => "ɾ",
            Self::RTap2 => "ɽ",
            Self::Phi => "ɸ",
            Self::Beta => "β",
            Self::F => "f",
            Self::V => "v",
            Self::Theta => "θ",
            Self::Del => "ð",
            Self::S => "s",
            Self::Z => "z",
            Self::Shh => "ʃ",
            Self::Zhh => "ʒ",
            Self::Sh => "ʂ",
            Self::Zh => "ʐ",
            Self::Ch => "ç",
            Self::JCurl => "ʝ",
            Self::X => "x",
            Self::Y => "ɣ",
            Self::Xh => "χ",
            Self::Yr => "ʁ",
            Self::HhLong => "ħ",
            Self::Crook => "ʕ",
            Self::H => "h",
            Self::Hh => "ɦ",
            Self::LBelt => "ɬ",
            Self::Lezh => "ɮ",
            Self::VHook => "ʋ",
            Self::RTilt => "ɹ",
            Self::RTiltHook => "ɻ",
            Self::J => "j",
            Self::MTiltTail => "ɰ",
            Self::L => "l",
            Self::Ll => "ɭ",
            Self::Lambda => "ʎ",
            Self::LCap => "ʟ",
        }
    }

    pub fn all() -> &'static [Self] {
        &ALL_CONSONANTS
    }
}

impl TryFrom<char> for Consonant {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'p' => Ok(Self::P),
            'b' => Ok(Self::B),
            't' => Ok(Self::T),
            'd' => Ok(Self::D),
            'ʈ' => Ok(Self::TRetroflex),
            'ɖ' => Ok(Self::DRetroflex),
            'c' => Ok(Self::C),
            'ɟ' => Ok(Self::JPalatal),
            'k' => Ok(Self::K),
            'g' => Ok(Self::G),
            'q' => Ok(Self::Q),
            'ɢ' => Ok(Self::GUvular),
            'ʔ' => Ok(Self::GlottalStop),
            'm' => Ok(Self::MBiLabial),
            'ɱ' => Ok(Self::MLabioDental),
            'n' => Ok(Self::N),
            'ɳ' => Ok(Self::NRetroflex),
            'ɲ' => Ok(Self::NPalatal),
            'ŋ' => Ok(Self::NVelar),
            'ɴ' => Ok(Self::NUvular),
            'ʙ' => Ok(Self::Brr),
            'r' => Ok(Self::Rrr),
            'ʀ' => Ok(Self::Rhrr),
            'ⱱ' => Ok(Self::VTap),
            'ɾ' => Ok(Self::RTap),
            'ɽ' => Ok(Self::RTap2),
            'ɸ' => Ok(Self::Phi),
            'β' => Ok(Self::Beta),
            'f' => Ok(Self::F),
            'v' => Ok(Self::V),
            'θ' => Ok(Self::Theta),
            'ð' => Ok(Self::Del),
            's' => Ok(Self::S),
            'z' => Ok(Self::Z),
            'ʃ' => Ok(Self::Shh),
            'ʒ' => Ok(Self::Zhh),
            'ʂ' => Ok(Self::Sh),
            'ʐ' => Ok(Self::Zh),
            'ç' => Ok(Self::Ch),
            'ʝ' => Ok(Self::JCurl),
            'x' => Ok(Self::X),
            'ɣ' => Ok(Self::Y),
            'χ' => Ok(Self::Xh),
            'ʁ' => Ok(Self::Yr),
            'ħ' => Ok(Self::HhLong),
            'ʕ' => Ok(Self::Crook),
            'h' => Ok(Self::H),
            'ɦ' => Ok(Self::Hh),
            'ɬ' => Ok(Self::LBelt),
            'ɮ' => Ok(Self::Lezh),
            'ʋ' => Ok(Self::VHook),
            'ɹ' => Ok(Self::RTilt),
            'ɻ' => Ok(Self::RTiltHook),
            'j' => Ok(Self::J),
            'ɰ' => Ok(Self::MTiltTail),
            'l' => Ok(Self::L),
            'ɭ' => Ok(Self::Ll),
            'ʎ' => Ok(Self::Lambda),
            'ʟ' => Ok(Self::LCap),
            _ => Err(ParseError::UnknownCharacter(value)),
        }
    }
}

impl FromStr for Consonant {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.len() {
            0 => Err(ParseError::NoInput),
            1 => Self::try_from(value.chars().nth(0).unwrap()),
            _ => Err(ParseError::TooManyCharacters),
        }
    }
}

impl fmt::Display for Consonant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl fmt::Debug for Consonant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Vowel {
    I,                     // <V- i
    Y,                     // <V- y
    IBar,                  // <V- ɨ
    UBar,                  // <V- ʉ
    Uu,                    // <V- ɯ
    U,                     // <V- u
    Ii,                    // <V- ɪ
    YCap,                  // <V- ʏ
    OmegaFlip,             // <V- ʊ
    E,                     // <V- e
    OCross,                // <V- ø
    EReverse,              // <V- ɘ
    OBar,                  // <V- ɵ
    RamsHorns,             // <V- ɤ
    O,                     // <V- o
    Schwa,                 // <V- ə
    EOpen,                 // <V- ɛ
    Oe,                    // <V- œ
    Ze,                    // <V- ɜ
    EpsilonClosedReversed, // <V- ɞ
    VFlip,                 // <V- ʌ
    OOpen,                 // <V- ɔ
    Ae,                    // <V- æ
    AFlip,                 // <V- ɐ
    A,                     // <V- a
    OeSmall,               // <V- ɶ
    AScript,               // <V- ɑ
    AScriptFlip,           // <V- ɒ
}

const ALL_VOWELS: [Vowel; 28] = [
    Vowel::I,
    Vowel::Y,
    Vowel::IBar,
    Vowel::UBar,
    Vowel::Uu,
    Vowel::U,
    Vowel::Ii,
    Vowel::YCap,
    Vowel::OmegaFlip,
    Vowel::E,
    Vowel::OCross,
    Vowel::EReverse,
    Vowel::OBar,
    Vowel::RamsHorns,
    Vowel::O,
    Vowel::Schwa,
    Vowel::EOpen,
    Vowel::Oe,
    Vowel::Ze,
    Vowel::EpsilonClosedReversed,
    Vowel::VFlip,
    Vowel::OOpen,
    Vowel::Ae,
    Vowel::AFlip,
    Vowel::A,
    Vowel::OeSmall,
    Vowel::AScript,
    Vowel::AScriptFlip,
];

impl Vowel {
    pub fn code(&self) -> &'static str {
        match self {
            Self::I => "i",
            Self::Y => "y",
            Self::IBar => "ɨ",
            Self::UBar => "ʉ",
            Self::Uu => "ɯ",
            Self::U => "u",
            Self::Ii => "ɪ",
            Self::YCap => "ʏ",
            Self::OmegaFlip => "ʊ",
            Self::E => "e",
            Self::OCross => "ø",
            Self::EReverse => "ɘ",
            Self::OBar => "ɵ",
            Self::RamsHorns => "ɤ",
            Self::O => "o",
            Self::Schwa => "ə",
            Self::EOpen => "ɛ",
            Self::Oe => "œ",
            Self::Ze => "ɜ",
            Self::EpsilonClosedReversed => "ɞ",
            Self::VFlip => "ʌ",
            Self::OOpen => "ɔ",
            Self::Ae => "æ",
            Self::AFlip => "ɐ",
            Self::A => "a",
            Self::OeSmall => "ɶ",
            Self::AScript => "ɑ",
            Self::AScriptFlip => "ɒ",
        }
    }

    pub fn all() -> &'static [Self] {
        &ALL_VOWELS
    }
}

impl fmt::Display for Vowel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl fmt::Debug for Vowel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl TryFrom<char> for Vowel {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'i' => Ok(Self::I),
            'y' => Ok(Self::Y),
            'ɨ' => Ok(Self::IBar),
            'ʉ' => Ok(Self::UBar),
            'ɯ' => Ok(Self::Uu),
            'u' => Ok(Self::U),
            'ɪ' => Ok(Self::Ii),
            'ʏ' => Ok(Self::YCap),
            'ʊ' => Ok(Self::OmegaFlip),
            'e' => Ok(Self::E),
            'ø' => Ok(Self::OCross),
            'ɘ' => Ok(Self::EReverse),
            'ɵ' => Ok(Self::OBar),
            'ɤ' => Ok(Self::RamsHorns),
            'o' => Ok(Self::O),
            'ə' => Ok(Self::Schwa),
            'ɛ' => Ok(Self::EOpen),
            'œ' => Ok(Self::Oe),
            'ɜ' => Ok(Self::Ze),
            'ɞ' => Ok(Self::EpsilonClosedReversed),
            'ʌ' => Ok(Self::VFlip),
            'ɔ' => Ok(Self::OOpen),
            'æ' => Ok(Self::Ae),
            'ɐ' => Ok(Self::AFlip),
            'a' => Ok(Self::A),
            'ɶ' => Ok(Self::OeSmall),
            'ɑ' => Ok(Self::AScript),
            'ɒ' => Ok(Self::AScriptFlip),
            _ => Err(ParseError::UnknownCharacter(value)),
        }
    }
}

impl FromStr for Vowel {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.len() {
            0 => Err(ParseError::NoInput),
            1 => Self::try_from(value.chars().nth(0).unwrap()),
            _ => Err(ParseError::TooManyCharacters),
        }
    }
}

/// Represents one of the phoneme types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phoneme {
    Consonant(Consonant),
    Vowel(Vowel),
}

impl From<Consonant> for Phoneme {
    fn from(value: Consonant) -> Self {
        Self::Consonant(value)
    }
}

impl From<Vowel> for Phoneme {
    fn from(value: Vowel) -> Self {
        Self::Vowel(value)
    }
}

impl Phoneme {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Consonant(c) => c.code(),
            Self::Vowel(v) => v.code(),
        }
    }
}

impl fmt::Display for Phoneme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl TryFrom<char> for Phoneme {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Consonant::try_from(value)
            .map(Into::into)
            .or_else(|_| Vowel::try_from(value).map(Into::into))
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    consonants: Vec<Consonant>,
    vowels: Vec<Vowel>,
}

impl Inventory {
    pub fn new(consonants: impl Into<Vec<Consonant>>, vowels: impl Into<Vec<Vowel>>) -> Self {
        Self {
            consonants: consonants.into(),
            vowels: vowels.into(),
        }
    }

    pub fn consonants(&self) -> &[Consonant] {
        &self.consonants
    }

    pub fn vowels(&self) -> &[Vowel] {
        &self.vowels
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.consonants.iter() {
            f.write_str(c.code())?;
        }
        f.write_str(" ")?;
        for v in self.vowels.iter() {
            f.write_str(v.code())?;
        }
        Ok(())
    }
}

fn parse_all<T>(source: &str) -> Result<Vec<T>, (Vec<T>, Vec<char>)>
where
    T: TryFrom<char, Error = ParseError>,
{
    let mut out = Vec::with_capacity(source.len());
    let mut unknown = Vec::new();

    for c in source.chars() {
        match T::try_from(c) {
            Ok(x) => out.push(x),
            Err(_) => unknown.push(c),
        }
    }

    if unknown.is_empty() {
        Ok(out)
    } else {
        Err((out, unknown))
    }
}

impl<'a> TryFrom<&'a str> for Inventory {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let Some(space_idx) = value.find(' ') else {
            return Err("No space in input".into());
        };
        let (consts_str, vowels_str) = value.split_at(space_idx);
        let vowels_str = &vowels_str[1..];

        let mut out_err = String::new();

        let consonants = match parse_all(consts_str) {
            Ok(x) => x,
            Err((x, unknowns)) => {
                write!(out_err, "unknown consonants: {:?}", unknowns).unwrap();
                x
            }
        };
        let vowels = match parse_all(vowels_str) {
            Ok(x) => x,
            Err((x, unknowns)) => {
                write!(out_err, "unknown vowels: {:?}", unknowns).unwrap();
                x
            }
        };

        if out_err.is_empty() {
            Ok(Inventory { consonants, vowels })
        } else {
            Err(out_err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consonants_parsing() {
        for orig in Consonant::all() {
            let code = orig.code();
            let parsed = Consonant::from_str(code).unwrap();
            assert_eq!(*orig, parsed);
        }
    }

    #[test]
    fn vowels_parsing() {
        for orig in Vowel::all() {
            let code = orig.code();
            let parsed = Vowel::from_str(code).unwrap();
            assert_eq!(*orig, parsed);
        }
    }
}
