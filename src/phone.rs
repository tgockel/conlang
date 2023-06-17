//! # Phonetics
//!
//! Phonetics form the basis of spoken language. This module contains `Phoneme`s as the basic building block of the
//! language and `Syllable`s to tie them together.

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
    GCap,        // <C- ɢ
    GlottalStop, // <C- ʔ
    // == Nasal ==
    M,          // <C- m
    MHook,      // <C- ɱ
    N,          // <C- n
    NRetroflex, // <C- ɳ
    NPalatal,   // <C- ɲ
    NVelar,     // <C- ŋ
    NUvular,    // <C- ɴ
    // == Trill ==
    BCap, // <C- ʙ
    Rrr,  // <C- r
    RCap, // <C- ʀ
    // == Tap or Flap ==
    VTap,  // <C- ⱱ
    RTap,  // <C- ɾ
    RFlap, // <C- ɽ
    // == Fricative ==
    Phi,   // <C- ɸ
    Beta,  // <C- β
    F,     // <C- f
    V,     // <C- v
    Theta, // <C- θ
    Del,   // <C- ð
    S,     // <C- s
    Z,     // <C- z
    Esh,   // <C- ʃ
    Ezh,   // <C- ʒ
    Sh,    // <C- ʂ
    Zh,    // <C- ʐ
    Ch,    // <C- ç
    JCurl, // <C- ʝ
    X,     // <C- x
    Gamma, // <C- ɣ
    Xh,    // <C- χ
    Yr,    // <C- ʁ
    HBar,  // <C- ħ
    Crook, // <C- ʕ
    H,     // <C- h
    HCurl, // <C- ɦ
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
    Consonant::GCap,
    Consonant::GlottalStop,
    Consonant::M,
    Consonant::MHook,
    Consonant::N,
    Consonant::NRetroflex,
    Consonant::NPalatal,
    Consonant::NVelar,
    Consonant::NUvular,
    Consonant::BCap,
    Consonant::Rrr,
    Consonant::RCap,
    Consonant::VTap,
    Consonant::RTap,
    Consonant::RFlap,
    Consonant::Phi,
    Consonant::Beta,
    Consonant::F,
    Consonant::V,
    Consonant::Theta,
    Consonant::Del,
    Consonant::S,
    Consonant::Z,
    Consonant::Esh,
    Consonant::Ezh,
    Consonant::Sh,
    Consonant::Zh,
    Consonant::Ch,
    Consonant::JCurl,
    Consonant::X,
    Consonant::Gamma,
    Consonant::Xh,
    Consonant::Yr,
    Consonant::HBar,
    Consonant::Crook,
    Consonant::H,
    Consonant::HCurl,
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
    pub fn code(&self) -> char {
        match self {
            Self::P => 'p',
            Self::B => 'b',
            Self::T => 't',
            Self::D => 'd',
            Self::TRetroflex => 'ʈ',
            Self::DRetroflex => 'ɖ',
            Self::C => 'c',
            Self::JPalatal => 'ɟ',
            Self::K => 'k',
            Self::G => 'g',
            Self::Q => 'q',
            Self::GCap => 'ɢ',
            Self::GlottalStop => 'ʔ',
            Self::M => 'm',
            Self::MHook => 'ɱ',
            Self::N => 'n',
            Self::NRetroflex => 'ɳ',
            Self::NPalatal => 'ɲ',
            Self::NVelar => 'ŋ',
            Self::NUvular => 'ɴ',
            Self::BCap => 'ʙ',
            Self::Rrr => 'r',
            Self::RCap => 'ʀ',
            Self::VTap => 'ⱱ',
            Self::RTap => 'ɾ',
            Self::RFlap => 'ɽ',
            Self::Phi => 'ɸ',
            Self::Beta => 'β',
            Self::F => 'f',
            Self::V => 'v',
            Self::Theta => 'θ',
            Self::Del => 'ð',
            Self::S => 's',
            Self::Z => 'z',
            Self::Esh => 'ʃ',
            Self::Ezh => 'ʒ',
            Self::Sh => 'ʂ',
            Self::Zh => 'ʐ',
            Self::Ch => 'ç',
            Self::JCurl => 'ʝ',
            Self::X => 'x',
            Self::Gamma => 'ɣ',
            Self::Xh => 'χ',
            Self::Yr => 'ʁ',
            Self::HBar => 'ħ',
            Self::Crook => 'ʕ',
            Self::H => 'h',
            Self::HCurl => 'ɦ',
            Self::LBelt => 'ɬ',
            Self::Lezh => 'ɮ',
            Self::VHook => 'ʋ',
            Self::RTilt => 'ɹ',
            Self::RTiltHook => 'ɻ',
            Self::J => 'j',
            Self::MTiltTail => 'ɰ',
            Self::L => 'l',
            Self::Ll => 'ɭ',
            Self::Lambda => 'ʎ',
            Self::LCap => 'ʟ',
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
            'ɢ' => Ok(Self::GCap),
            'ʔ' => Ok(Self::GlottalStop),
            'm' => Ok(Self::M),
            'ɱ' => Ok(Self::MHook),
            'n' => Ok(Self::N),
            'ɳ' => Ok(Self::NRetroflex),
            'ɲ' => Ok(Self::NPalatal),
            'ŋ' => Ok(Self::NVelar),
            'ɴ' => Ok(Self::NUvular),
            'ʙ' => Ok(Self::BCap),
            'r' => Ok(Self::Rrr),
            'ʀ' => Ok(Self::RCap),
            'ⱱ' => Ok(Self::VTap),
            'ɾ' => Ok(Self::RTap),
            'ɽ' => Ok(Self::RFlap),
            'ɸ' => Ok(Self::Phi),
            'β' => Ok(Self::Beta),
            'f' => Ok(Self::F),
            'v' => Ok(Self::V),
            'θ' => Ok(Self::Theta),
            'ð' => Ok(Self::Del),
            's' => Ok(Self::S),
            'z' => Ok(Self::Z),
            'ʃ' => Ok(Self::Esh),
            'ʒ' => Ok(Self::Ezh),
            'ʂ' => Ok(Self::Sh),
            'ʐ' => Ok(Self::Zh),
            'ç' => Ok(Self::Ch),
            'ʝ' => Ok(Self::JCurl),
            'x' => Ok(Self::X),
            'ɣ' => Ok(Self::Gamma),
            'χ' => Ok(Self::Xh),
            'ʁ' => Ok(Self::Yr),
            'ħ' => Ok(Self::HBar),
            'ʕ' => Ok(Self::Crook),
            'h' => Ok(Self::H),
            'ɦ' => Ok(Self::HCurl),
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
        match value.chars().count() {
            0 => Err(ParseError::NoInput),
            1 => Self::try_from(value.chars().nth(0).unwrap()),
            _ => Err(ParseError::TooManyCharacters),
        }
    }
}

impl fmt::Display for Consonant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
    }
}

impl fmt::Debug for Consonant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
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
    pub fn code(&self) -> char {
        match self {
            Self::I => 'i',
            Self::Y => 'y',
            Self::IBar => 'ɨ',
            Self::UBar => 'ʉ',
            Self::Uu => 'ɯ',
            Self::U => 'u',
            Self::Ii => 'ɪ',
            Self::YCap => 'ʏ',
            Self::OmegaFlip => 'ʊ',
            Self::E => 'e',
            Self::OCross => 'ø',
            Self::EReverse => 'ɘ',
            Self::OBar => 'ɵ',
            Self::RamsHorns => 'ɤ',
            Self::O => 'o',
            Self::Schwa => 'ə',
            Self::EOpen => 'ɛ',
            Self::Oe => 'œ',
            Self::Ze => 'ɜ',
            Self::EpsilonClosedReversed => 'ɞ',
            Self::VFlip => 'ʌ',
            Self::OOpen => 'ɔ',
            Self::Ae => 'æ',
            Self::AFlip => 'ɐ',
            Self::A => 'a',
            Self::OeSmall => 'ɶ',
            Self::AScript => 'ɑ',
            Self::AScriptFlip => 'ɒ',
        }
    }

    pub fn all() -> &'static [Self] {
        &ALL_VOWELS
    }
}

impl fmt::Display for Vowel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
    }
}

impl fmt::Debug for Vowel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
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
        match value.chars().count() {
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
    pub fn code(&self) -> char {
        match self {
            Self::Consonant(c) => c.code(),
            Self::Vowel(v) => v.code(),
        }
    }
}

impl fmt::Display for Phoneme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
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

#[derive(Clone)]
enum SyllableInner {
    Local1([Phoneme; 1]),
    Local2([Phoneme; 2]),
    Local3([Phoneme; 3]),
    Local4([Phoneme; 4]),
    Heap(Vec<Phoneme>),
}

#[derive(Clone)]
pub struct Syllable {
    inner: SyllableInner,
}

impl Syllable {
    pub fn new(seq: &[Phoneme]) -> Self {
        let inner = match seq.len() {
            1 => SyllableInner::Local1([seq[0]]),
            2 => SyllableInner::Local2([seq[0], seq[1]]),
            3 => SyllableInner::Local3([seq[0], seq[1], seq[2]]),
            4 => SyllableInner::Local4([seq[0], seq[1], seq[2], seq[3]]),
            _ => SyllableInner::Heap(seq.into()),
        };
        Self { inner }
    }

    pub fn parts(&self) -> &[Phoneme] {
        match &self.inner {
            SyllableInner::Local1(d) => d,
            SyllableInner::Local2(d) => d,
            SyllableInner::Local3(d) => d,
            SyllableInner::Local4(d) => d,
            SyllableInner::Heap(d) => &d,
        }
    }
}

impl PartialEq for Syllable {
    fn eq(&self, other: &Self) -> bool {
        self.parts().len() == other.parts().len()
            && self
                .parts()
                .iter()
                .zip(other.parts().iter())
                .all(|(a, b)| a == b)
    }
}

impl Eq for Syllable {}

impl fmt::Display for Syllable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for p in self.parts() {
            write!(f, "{p}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Syllable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for Syllable {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ps = Vec::with_capacity(s.len() * 2);
        for c in s.chars() {
            ps.push(Phoneme::try_from(c)?);
        }
        Ok(Self::new(&ps))
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
            f.write_char(c.code())?;
        }
        f.write_char(' ')?;
        for v in self.vowels.iter() {
            f.write_char(v.code())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consonants_parsing() {
        for orig in Consonant::all() {
            let code = orig.code();
            let parsed = Consonant::from_str(&format!("{code}")).unwrap();
            assert_eq!(*orig, parsed);
        }
    }

    #[test]
    fn vowels_parsing() {
        for orig in Vowel::all() {
            let code = orig.code();
            let parsed = Vowel::from_str(&format!("{code}")).unwrap();
            assert_eq!(*orig, parsed);
        }
    }

    #[test]
    fn unique() {
        let vec: Vec<_> = Consonant::all()
            .iter()
            .map(Consonant::code)
            .chain(Vowel::all().iter().map(Vowel::code))
            .collect();
        let set: std::collections::BTreeSet<_> = vec.iter().collect();

        // There should not be any codes shared between phonic classes
        assert_eq!(vec.len(), set.len());
    }
}
