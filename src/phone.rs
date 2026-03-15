//! # Phonetics
//!
//! Phonetics form the basis of spoken language. This module contains `Phoneme`s as the basic building block of the
//! language and `Syllable`s to tie them together.

use bitflags::bitflags;
use smallvec::{SmallVec, smallvec};
use std::{
    error::Error,
    fmt::{self, Write},
    num::NonZeroU8,
    str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    NoInput,
    TooManyCharacters,
    UnknownCharacter(char),
    /// A tie bar (◌͡◌) was found without a following base phone.
    InvalidTieBar,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoInput => write!(f, "no input"),
            Self::TooManyCharacters => write!(f, "too many characters in input"),
            Self::UnknownCharacter(c) => write!(f, "unknown character '{c}'"),
            Self::InvalidTieBar => write!(f, "tie bar without following base phone"),
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
    pub fn all() -> &'static [Self] {
        &ALL_CONSONANTS
    }

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

    pub fn manner(&self) -> Manner {
        match self {
            Self::P => Manner::Plosive,
            Self::B => Manner::Plosive,
            Self::T => Manner::Plosive,
            Self::D => Manner::Plosive,
            Self::TRetroflex => Manner::Plosive,
            Self::DRetroflex => Manner::Plosive,
            Self::C => Manner::Plosive,
            Self::JPalatal => Manner::Plosive,
            Self::K => Manner::Plosive,
            Self::G => Manner::Plosive,
            Self::Q => Manner::Plosive,
            Self::GCap => Manner::Plosive,
            Self::GlottalStop => Manner::Plosive,
            Self::M => Manner::Nasal,
            Self::MHook => Manner::Nasal,
            Self::N => Manner::Nasal,
            Self::NRetroflex => Manner::Nasal,
            Self::NPalatal => Manner::Nasal,
            Self::NVelar => Manner::Nasal,
            Self::NUvular => Manner::Nasal,
            Self::BCap => Manner::Trill,
            Self::Rrr => Manner::Trill,
            Self::RCap => Manner::Trill,
            Self::VTap => Manner::Tap,
            Self::RTap => Manner::Tap,
            Self::RFlap => Manner::Tap,
            Self::Phi => Manner::Fricative,
            Self::Beta => Manner::Fricative,
            Self::F => Manner::Fricative,
            Self::V => Manner::Fricative,
            Self::Theta => Manner::Fricative,
            Self::Del => Manner::Fricative,
            Self::S => Manner::Fricative,
            Self::Z => Manner::Fricative,
            Self::Esh => Manner::Fricative,
            Self::Ezh => Manner::Fricative,
            Self::Sh => Manner::Fricative,
            Self::Zh => Manner::Fricative,
            Self::Ch => Manner::Fricative,
            Self::JCurl => Manner::Fricative,
            Self::X => Manner::Fricative,
            Self::Gamma => Manner::Fricative,
            Self::Xh => Manner::Fricative,
            Self::Yr => Manner::Fricative,
            Self::HBar => Manner::Fricative,
            Self::Crook => Manner::Fricative,
            Self::H => Manner::Fricative,
            Self::HCurl => Manner::Fricative,
            Self::LBelt => Manner::LateralFricative,
            Self::Lezh => Manner::LateralFricative,
            Self::VHook => Manner::Approximant,
            Self::RTilt => Manner::Approximant,
            Self::RTiltHook => Manner::Approximant,
            Self::J => Manner::Approximant,
            Self::MTiltTail => Manner::Approximant,
            Self::L => Manner::LateralApproximant,
            Self::Ll => Manner::LateralApproximant,
            Self::Lambda => Manner::LateralApproximant,
            Self::LCap => Manner::LateralApproximant,
        }
    }

    pub fn place(&self) -> Place {
        match self {
            Self::P => Place::Bilabial,
            Self::B => Place::Bilabial,
            Self::T => Place::Alveolar,
            Self::D => Place::Alveolar,
            Self::TRetroflex => Place::Retroflex,
            Self::DRetroflex => Place::Retroflex,
            Self::C => Place::Palatal,
            Self::JPalatal => Place::Palatal,
            Self::K => Place::Velar,
            Self::G => Place::Velar,
            Self::Q => Place::Uvular,
            Self::GCap => Place::Uvular,
            Self::GlottalStop => Place::Glottal,
            Self::M => Place::Bilabial,
            Self::MHook => Place::Labiodental,
            Self::N => Place::Alveolar,
            Self::NRetroflex => Place::Retroflex,
            Self::NPalatal => Place::Palatal,
            Self::NVelar => Place::Velar,
            Self::NUvular => Place::Uvular,
            Self::BCap => Place::Bilabial,
            Self::Rrr => Place::Alveolar,
            Self::RCap => Place::Uvular,
            Self::VTap => Place::Labiodental,
            Self::RTap => Place::Alveolar,
            Self::RFlap => Place::Retroflex,
            Self::Phi => Place::Bilabial,
            Self::Beta => Place::Bilabial,
            Self::F => Place::Labiodental,
            Self::V => Place::Labiodental,
            Self::Theta => Place::Dental,
            Self::Del => Place::Dental,
            Self::S => Place::Alveolar,
            Self::Z => Place::Alveolar,
            Self::Esh => Place::PostAlveolar,
            Self::Ezh => Place::PostAlveolar,
            Self::Sh => Place::Retroflex,
            Self::Zh => Place::Retroflex,
            Self::Ch => Place::Palatal,
            Self::JCurl => Place::Palatal,
            Self::X => Place::Velar,
            Self::Gamma => Place::Velar,
            Self::Xh => Place::Uvular,
            Self::Yr => Place::Uvular,
            Self::HBar => Place::Pharyngeal,
            Self::Crook => Place::Pharyngeal,
            Self::H => Place::Glottal,
            Self::HCurl => Place::Glottal,
            Self::LBelt => Place::Alveolar,
            Self::Lezh => Place::Alveolar,
            Self::VHook => Place::Labiodental,
            Self::RTilt => Place::Alveolar,
            Self::RTiltHook => Place::Retroflex,
            Self::J => Place::Palatal,
            Self::MTiltTail => Place::Velar,
            Self::L => Place::Alveolar,
            Self::Ll => Place::Retroflex,
            Self::Lambda => Place::Palatal,
            Self::LCap => Place::Velar,
        }
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
pub enum NonPulmonicConsonant {
    BilabialClick,     // <K- ʘ
    DentalClick,       // <K- ǀ
    Postalveoalar,     // <K- ǃ
    Palatoalveolar,    // <K- ǂ
    AlveolarLateral,   // <K- ǁ
    BilabialImplosive, // <K- ɓ
    DentalImplosive,   // <K- ɗ
    Palatal,           // <K- ʄ
    Velar,             // <K- ɠ
    Uvular,            // <K- ʛ
}

const ALL_NON_PULMONIC_CONSTANTS: [NonPulmonicConsonant; 10] = [
    NonPulmonicConsonant::BilabialClick,
    NonPulmonicConsonant::DentalClick,
    NonPulmonicConsonant::Postalveoalar,
    NonPulmonicConsonant::Palatoalveolar,
    NonPulmonicConsonant::AlveolarLateral,
    NonPulmonicConsonant::BilabialImplosive,
    NonPulmonicConsonant::DentalImplosive,
    NonPulmonicConsonant::Palatal,
    NonPulmonicConsonant::Velar,
    NonPulmonicConsonant::Uvular,
];

impl NonPulmonicConsonant {
    pub fn code(&self) -> char {
        match self {
            Self::BilabialClick => 'ʘ',
            Self::DentalClick => 'ǀ',
            Self::Postalveoalar => 'ǃ',
            Self::Palatoalveolar => 'ǂ',
            Self::AlveolarLateral => 'ǁ',
            Self::BilabialImplosive => 'ɓ',
            Self::DentalImplosive => 'ɗ',
            Self::Palatal => 'ʄ',
            Self::Velar => 'ɠ',
            Self::Uvular => 'ʛ',
        }
    }

    pub fn all() -> &'static [Self] {
        &ALL_NON_PULMONIC_CONSTANTS
    }
}

impl TryFrom<char> for NonPulmonicConsonant {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'ʘ' => Ok(Self::BilabialClick),
            'ǀ' => Ok(Self::DentalClick),
            'ǃ' => Ok(Self::Postalveoalar),
            'ǂ' => Ok(Self::Palatoalveolar),
            'ǁ' => Ok(Self::AlveolarLateral),
            'ɓ' => Ok(Self::BilabialImplosive),
            'ɗ' => Ok(Self::DentalImplosive),
            'ʄ' => Ok(Self::Palatal),
            'ɠ' => Ok(Self::Velar),
            'ʛ' => Ok(Self::Uvular),
            _ => Err(ParseError::UnknownCharacter(value)),
        }
    }
}

impl FromStr for NonPulmonicConsonant {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.chars().count() {
            0 => Err(ParseError::NoInput),
            1 => Self::try_from(value.chars().nth(0).unwrap()),
            _ => Err(ParseError::TooManyCharacters),
        }
    }
}

impl fmt::Display for NonPulmonicConsonant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
    }
}

impl fmt::Debug for NonPulmonicConsonant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(self.code())
    }
}

/// The [place of articulation](https://en.wikipedia.org/wiki/Place_of_articulation) is the location on the vocal tract
/// where the sound production occurs. More simply, this is a place in the mouth. A bilabial sound like "p" comes from
/// the lips, while a glottal sound like "h" comes from the throat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    Bilabial,
    Labiodental,
    Dental,
    Alveolar,
    PostAlveolar,
    Retroflex,
    Palatal,
    Velar,
    Uvular,
    Pharyngeal,
    Glottal,
}

impl Place {
    pub fn try_from(value: char) -> Result<SmallVec<[Self; 4]>, ParseError> {
        match value {
            'M' => Ok(smallvec![Self::Bilabial]),
            'L' => Ok(smallvec![Self::Labiodental]),
            'D' => Ok(smallvec![Self::Dental, Self::Alveolar, Self::PostAlveolar]),
            'Ḍ' => Ok(smallvec![Self::Retroflex]),
            'J' => Ok(smallvec![Self::Palatal]),
            'G' => Ok(smallvec![Self::Velar]),
            'Q' => Ok(smallvec![Self::Uvular]),
            'H' => Ok(smallvec![Self::Pharyngeal, Self::Glottal]),
            _ => Err(ParseError::UnknownCharacter(value)),
        }
    }
}

/// The [manner of articulation](https://en.wikipedia.org/wiki/Manner_of_articulation) is the interaction of the speech
/// organs used to make the sound. A plosive like "t" is a full stop of air, a nasal sound like "n" is made by allowing
/// air to escape through the nose, and a lateral approximate sound like "l" allows air to escape around the sides of
/// the tongue (note that these are all alveolar in place, yet sound different).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Manner {
    Plosive,
    Nasal,
    Trill,
    Tap,
    Fricative,
    LateralFricative,
    Approximant,
    LateralApproximant,
    Affricate,
}

impl Manner {
    pub fn try_from(value: char) -> Result<SmallVec<[Self; 2]>, ParseError> {
        match value {
            'P' => Ok(smallvec![Self::Plosive]),
            'N' => Ok(smallvec![Self::Nasal]),
            'T' => Ok(smallvec![Self::Trill]),
            'X' => Ok(smallvec![Self::Fricative, Self::LateralFricative]),
            'R' => Ok(smallvec![Self::Approximant, Self::LateralApproximant]),
            'A' => Ok(smallvec![Self::Affricate]),
            _ => Err(ParseError::UnknownCharacter(value)),
        }
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

    pub fn height(&self) -> Height {
        let value = match self {
            Self::I => 9,
            Self::Y => 9,
            Self::IBar => 9,
            Self::UBar => 9,
            Self::Uu => 9,
            Self::U => 9,
            Self::Ii => 8,
            Self::YCap => 8,
            Self::OmegaFlip => 8,
            Self::E => 7,
            Self::OCross => 7,
            Self::EReverse => 7,
            Self::OBar => 7,
            Self::RamsHorns => 7,
            Self::O => 7,
            Self::Schwa => 5,
            Self::EOpen => 3,
            Self::Oe => 3,
            Self::Ze => 3,
            Self::EpsilonClosedReversed => 3,
            Self::VFlip => 3,
            Self::OOpen => 3,
            Self::Ae => 2,
            Self::AFlip => 2,
            Self::A => 1,
            Self::OeSmall => 1,
            Self::AScript => 1,
            Self::AScriptFlip => 1,
        };
        Height::new(value)
    }

    pub fn frontness(&self) -> Frontness {
        let value = match self {
            Self::I => 9,
            Self::Y => 9,
            Self::IBar => 5,
            Self::UBar => 5,
            Self::Uu => 1,
            Self::U => 1,
            Self::Ii => 8,
            Self::YCap => 8,
            Self::OmegaFlip => 1,
            Self::E => 9,
            Self::OCross => 9,
            Self::EReverse => 5,
            Self::OBar => 5,
            Self::RamsHorns => 1,
            Self::O => 1,
            Self::Schwa => 5,
            Self::EOpen => 8,
            Self::Oe => 8,
            Self::Ze => 4,
            Self::EpsilonClosedReversed => 4,
            Self::VFlip => 1,
            Self::OOpen => 1,
            Self::Ae => 7,
            Self::AFlip => 3,
            Self::A => 6,
            Self::OeSmall => 6,
            Self::AScript => 1,
            Self::AScriptFlip => 1,
        };
        Frontness::new(value)
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

/// For a `Vowel`, how high the tongue is in the mouth. The value has a range 1 to 9, inclusive.
///
/// The International Phonetic Alphabet classifies vowel sounds into "Close," "Close-Mid," "Open-Mid," and "Open." An
/// `enum` for this is not sufficient, since characters like "near-close near-front unrounded vowel" (ɪ) and "near-open
/// central vowel" (ɐ) land in between these categories. Values of 1 represent fully-open vowels like a and ɒ; values of
/// 9 represent the fully-closed vowels like i and ɯ; and other values somewhere in between.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Height {
    value: NonZeroU8,
}

impl Height {
    pub fn new(value: u8) -> Self {
        assert!(value > 0);
        assert!(value <= 9);
        let value = unsafe { NonZeroU8::new_unchecked(value) };
        Self { value }
    }

    pub fn value(&self) -> u8 {
        self.value.get()
    }
}

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

/// For a `Vowel`, how close the tongue is to the front of the mouth. The value has a range 0 to 10, inclusive.
///
/// The International Phonetic Alphabet has vowels on a slant, since higher vowels allow the tongue more maneuverability
/// forward (presumably your teeth get in the way for open vowels). This does not correct for that; in other words, the
/// trapezoid shape is not considered in frontness values. So the "open front unrounded vowel" (a) has a value of 6,
/// despite being the most front an open vowel can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Frontness {
    value: NonZeroU8,
}

impl Frontness {
    pub fn new(value: u8) -> Self {
        assert!(value > 0);
        assert!(value <= 9);
        let value = unsafe { NonZeroU8::new_unchecked(value) };
        Self { value }
    }

    pub fn value(&self) -> u8 {
        self.value.get()
    }
}

impl fmt::Display for Frontness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

bitflags! {
    /// Diacritical modifiers that can be applied to a base phone.
    ///
    /// Each flag corresponds to a single IPA combining character or modifier letter.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Diacritics: u32 {
        // Phonation
        const ASPIRATED       = 1 << 0;  // ʰ (U+02B0)
        const VOICELESS       = 1 << 1;  // ̥  (U+0325)
        const VOICED          = 1 << 2;  // ̬  (U+032C)
        const BREATHY_VOICE   = 1 << 3;  // ̤  (U+0324)
        const CREAKY_VOICE    = 1 << 4;  // ̰  (U+0330)

        // Secondary articulation
        const LABIALIZED      = 1 << 5;  // ʷ (U+02B7)
        const PALATALIZED     = 1 << 6;  // ʲ (U+02B2)
        const VELARIZED       = 1 << 7;  // ˠ (U+02E0)
        const PHARYNGEALIZED  = 1 << 8;  // ˤ (U+02E4)
        const NASALIZED       = 1 << 9;  // ̃  (U+0303)

        // Length
        const LONG            = 1 << 10; // ː (U+02D0)
        const HALF_LONG       = 1 << 11; // ˑ (U+02D1)

        // Place refinement
        const DENTAL          = 1 << 12; // ̪  (U+032A)
        const APICAL          = 1 << 13; // ̺  (U+033A)
        const LAMINAL         = 1 << 14; // ̻  (U+033B)
        const ADVANCED        = 1 << 15; // ̟  (U+031F)
        const RETRACTED       = 1 << 16; // ̠  (U+0320)

        // Vowel refinement
        const MORE_ROUNDED    = 1 << 17; // ̹  (U+0339)
        const LESS_ROUNDED    = 1 << 18; // ̜  (U+031C)
        const RAISED          = 1 << 19; // ̝  (U+031D)
        const LOWERED         = 1 << 20; // ̞  (U+031E)

        // Release
        const NO_AUDIBLE_RELEASE = 1 << 21; // ̚ (U+031A)
        const LATERAL_RELEASE    = 1 << 22; // ˡ (U+02E1)
        const NASAL_RELEASE      = 1 << 23; // ⁿ (U+207F)

        // Syllabicity
        const SYLLABIC           = 1 << 24; // ̩ (U+0329)
        const NON_SYLLABIC       = 1 << 25; // ̯ (U+032F)

        // Other
        const RHOTACIZED         = 1 << 26; // ˞ (U+02DE)
        const CENTRALIZED        = 1 << 27; // ̈  (U+0308)
    }
}

impl Diacritics {
    /// Try to parse a single character as a diacritic modifier.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'ʰ' => Some(Self::ASPIRATED),
            '\u{0325}' => Some(Self::VOICELESS),
            '\u{032C}' => Some(Self::VOICED),
            '\u{0324}' => Some(Self::BREATHY_VOICE),
            '\u{0330}' => Some(Self::CREAKY_VOICE),
            'ʷ' => Some(Self::LABIALIZED),
            'ʲ' => Some(Self::PALATALIZED),
            'ˠ' => Some(Self::VELARIZED),
            'ˤ' => Some(Self::PHARYNGEALIZED),
            '\u{0303}' => Some(Self::NASALIZED),
            'ː' => Some(Self::LONG),
            'ˑ' => Some(Self::HALF_LONG),
            '\u{032A}' => Some(Self::DENTAL),
            '\u{033A}' => Some(Self::APICAL),
            '\u{033B}' => Some(Self::LAMINAL),
            '\u{031F}' => Some(Self::ADVANCED),
            '\u{0320}' => Some(Self::RETRACTED),
            '\u{0339}' => Some(Self::MORE_ROUNDED),
            '\u{031C}' => Some(Self::LESS_ROUNDED),
            '\u{031D}' => Some(Self::RAISED),
            '\u{031E}' => Some(Self::LOWERED),
            '\u{031A}' => Some(Self::NO_AUDIBLE_RELEASE),
            'ˡ' => Some(Self::LATERAL_RELEASE),
            'ⁿ' => Some(Self::NASAL_RELEASE),
            '\u{0329}' => Some(Self::SYLLABIC),
            '\u{032F}' => Some(Self::NON_SYLLABIC),
            '\u{02DE}' => Some(Self::RHOTACIZED),
            '\u{0308}' => Some(Self::CENTRALIZED),
            _ => None,
        }
    }

    /// Emit the IPA characters for the set diacritics in canonical order.
    ///
    /// Order: combining marks (below, then above), then modifier letters, then length.
    fn write_ipa(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Combining marks below
        if self.contains(Self::VOICELESS) {
            f.write_char('\u{0325}')?;
        }
        if self.contains(Self::VOICED) {
            f.write_char('\u{032C}')?;
        }
        if self.contains(Self::BREATHY_VOICE) {
            f.write_char('\u{0324}')?;
        }
        if self.contains(Self::CREAKY_VOICE) {
            f.write_char('\u{0330}')?;
        }
        if self.contains(Self::DENTAL) {
            f.write_char('\u{032A}')?;
        }
        if self.contains(Self::APICAL) {
            f.write_char('\u{033A}')?;
        }
        if self.contains(Self::LAMINAL) {
            f.write_char('\u{033B}')?;
        }
        if self.contains(Self::ADVANCED) {
            f.write_char('\u{031F}')?;
        }
        if self.contains(Self::RETRACTED) {
            f.write_char('\u{0320}')?;
        }
        if self.contains(Self::MORE_ROUNDED) {
            f.write_char('\u{0339}')?;
        }
        if self.contains(Self::LESS_ROUNDED) {
            f.write_char('\u{031C}')?;
        }
        if self.contains(Self::RAISED) {
            f.write_char('\u{031D}')?;
        }
        if self.contains(Self::LOWERED) {
            f.write_char('\u{031E}')?;
        }
        if self.contains(Self::NO_AUDIBLE_RELEASE) {
            f.write_char('\u{031A}')?;
        }
        if self.contains(Self::SYLLABIC) {
            f.write_char('\u{0329}')?;
        }
        if self.contains(Self::NON_SYLLABIC) {
            f.write_char('\u{032F}')?;
        }
        // Combining marks above
        if self.contains(Self::NASALIZED) {
            f.write_char('\u{0303}')?;
        }
        if self.contains(Self::CENTRALIZED) {
            f.write_char('\u{0308}')?;
        }
        // Modifier letters
        if self.contains(Self::ASPIRATED) {
            f.write_char('ʰ')?;
        }
        if self.contains(Self::LABIALIZED) {
            f.write_char('ʷ')?;
        }
        if self.contains(Self::PALATALIZED) {
            f.write_char('ʲ')?;
        }
        if self.contains(Self::VELARIZED) {
            f.write_char('ˠ')?;
        }
        if self.contains(Self::PHARYNGEALIZED) {
            f.write_char('ˤ')?;
        }
        if self.contains(Self::LATERAL_RELEASE) {
            f.write_char('ˡ')?;
        }
        if self.contains(Self::NASAL_RELEASE) {
            f.write_char('ⁿ')?;
        }
        if self.contains(Self::RHOTACIZED) {
            f.write_char('\u{02DE}')?;
        }
        // Length (last)
        if self.contains(Self::LONG) {
            f.write_char('ː')?;
        }
        if self.contains(Self::HALF_LONG) {
            f.write_char('ˑ')?;
        }
        Ok(())
    }
}

impl fmt::Display for Diacritics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write_ipa(f)
    }
}

/// A single phonetic segment: one base phone with optional diacritics,
/// or two base phones joined by a tie bar (affricate / co-articulation).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Segment {
    Simple {
        base: Phoneme,
        diacritics: Diacritics,
    },
    Compound {
        first: Phoneme,
        second: Phoneme,
        diacritics: Diacritics,
    },
}

impl Segment {
    /// Create a simple segment from a base phoneme with no diacritics.
    pub fn simple(base: Phoneme) -> Self {
        Self::Simple {
            base,
            diacritics: Diacritics::empty(),
        }
    }

    /// Create a simple segment with diacritics.
    pub fn with_diacritics(base: Phoneme, diacritics: Diacritics) -> Self {
        Self::Simple { base, diacritics }
    }

    /// Create a compound segment (affricate or co-articulation).
    pub fn compound(first: Phoneme, second: Phoneme) -> Self {
        Self::Compound {
            first,
            second,
            diacritics: Diacritics::empty(),
        }
    }

    /// Create a compound segment with diacritics.
    pub fn compound_with_diacritics(
        first: Phoneme,
        second: Phoneme,
        diacritics: Diacritics,
    ) -> Self {
        Self::Compound {
            first,
            second,
            diacritics,
        }
    }

    /// The base phoneme (for Simple) or first component (for Compound).
    pub fn base(&self) -> Phoneme {
        match self {
            Self::Simple { base, .. } => *base,
            Self::Compound { first, .. } => *first,
        }
    }

    /// Whether this segment acts as a consonant in syllable structure.
    pub fn is_consonantal(&self) -> bool {
        match self {
            Self::Simple { base, .. } => matches!(
                base,
                Phoneme::Consonant(_) | Phoneme::NonPulmonicConsonant(_)
            ),
            Self::Compound { .. } => true,
        }
    }

    /// Whether this segment acts as a vowel in syllable structure.
    pub fn is_vocalic(&self) -> bool {
        matches!(
            self,
            Self::Simple {
                base: Phoneme::Vowel(_),
                ..
            }
        )
    }

    /// Place of articulation, if this is a consonantal segment.
    /// For compounds, returns the place of the first element.
    pub fn place(&self) -> Option<Place> {
        match self {
            Self::Simple {
                base: Phoneme::Consonant(c),
                ..
            } => Some(c.place()),
            Self::Compound {
                first: Phoneme::Consonant(c),
                ..
            } => Some(c.place()),
            _ => None,
        }
    }

    /// Manner of articulation.
    /// For compounds, returns `Manner::Affricate`.
    pub fn manner(&self) -> Option<Manner> {
        match self {
            Self::Simple {
                base: Phoneme::Consonant(c),
                ..
            } => Some(c.manner()),
            Self::Compound { .. } => Some(Manner::Affricate),
            _ => None,
        }
    }

    pub fn diacritics(&self) -> Diacritics {
        match self {
            Self::Simple { diacritics, .. } | Self::Compound { diacritics, .. } => *diacritics,
        }
    }

    pub fn has_diacritic(&self, d: Diacritics) -> bool {
        self.diacritics().contains(d)
    }

    /// Parse a single segment from the beginning of an IPA string.
    /// Returns the parsed segment and the remaining unparsed input.
    pub fn parse_ipa(input: &str) -> Result<(Self, &str), ParseError> {
        let mut chars = input.char_indices();

        // Read base character
        let (_, first_char) = chars.next().ok_or(ParseError::NoInput)?;
        let base = Phoneme::try_from(first_char)?;

        // Peek at next char: tie bar?
        let after_base = chars.clone();
        if let Some((_, '\u{0361}')) = chars.next() {
            // Compound segment: read second base
            let (_, second_char) = chars.next().ok_or(ParseError::InvalidTieBar)?;
            let second = Phoneme::try_from(second_char)?;

            // Collect diacritics
            let mut diacritics = Diacritics::empty();
            loop {
                let checkpoint = chars.clone();
                match chars.next() {
                    Some((_, c)) => match Diacritics::from_char(c) {
                        Some(d) => diacritics |= d,
                        None => {
                            // Not a diacritic; put it back
                            chars = checkpoint;
                            break;
                        }
                    },
                    None => break,
                }
            }

            let remaining = match chars.next() {
                Some((idx, _)) => &input[idx..],
                None => "",
            };
            Ok((
                Self::Compound {
                    first: base,
                    second,
                    diacritics,
                },
                remaining,
            ))
        } else {
            // Simple segment: collect diacritics
            let mut chars = after_base;
            let mut diacritics = Diacritics::empty();
            loop {
                let checkpoint = chars.clone();
                match chars.next() {
                    Some((_, c)) => match Diacritics::from_char(c) {
                        Some(d) => diacritics |= d,
                        None => {
                            chars = checkpoint;
                            break;
                        }
                    },
                    None => break,
                }
            }

            let remaining = match chars.next() {
                Some((idx, _)) => &input[idx..],
                None => "",
            };
            Ok((Self::Simple { base, diacritics }, remaining))
        }
    }
}

impl From<Phoneme> for Segment {
    fn from(p: Phoneme) -> Self {
        Self::simple(p)
    }
}

impl From<Consonant> for Segment {
    fn from(c: Consonant) -> Self {
        Self::simple(Phoneme::Consonant(c))
    }
}

impl From<Vowel> for Segment {
    fn from(v: Vowel) -> Self {
        Self::simple(Phoneme::Vowel(v))
    }
}

impl From<NonPulmonicConsonant> for Segment {
    fn from(c: NonPulmonicConsonant) -> Self {
        Self::simple(Phoneme::NonPulmonicConsonant(c))
    }
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Simple { base, diacritics } => {
                write!(f, "{base}")?;
                diacritics.write_ipa(f)?;
            }
            Self::Compound {
                first,
                second,
                diacritics,
            } => {
                write!(f, "{first}\u{0361}{second}")?;
                diacritics.write_ipa(f)?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Stress level for a syllable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stress {
    #[default]
    None,
    /// Primary stress (ˈ, U+02C8).
    Primary,
    /// Secondary stress (ˌ, U+02CC).
    Secondary,
}

impl Stress {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'ˈ' => Some(Self::Primary),
            'ˌ' => Some(Self::Secondary),
            _ => None,
        }
    }
}

impl fmt::Display for Stress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Primary => f.write_char('ˈ'),
            Self::Secondary => f.write_char('ˌ'),
        }
    }
}

/// Tone contour represented as a sequence of tone levels.
///
/// Each level is 1-5, mapping to IPA tone letters ˩˨˧˦˥.
/// An empty tone means no tone marking.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Tone {
    levels: SmallVec<[u8; 3]>,
}

impl Tone {
    /// Create a level (register) tone.
    pub fn level(n: u8) -> Self {
        assert!((1..=5).contains(&n));
        Self {
            levels: smallvec![n],
        }
    }

    /// Create a contour tone from a sequence of levels.
    pub fn contour(ns: &[u8]) -> Self {
        for &n in ns {
            assert!((1..=5).contains(&n));
        }
        Self {
            levels: SmallVec::from(ns),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    fn tone_letter(level: u8) -> char {
        match level {
            1 => '˩', // U+02E9
            2 => '˨', // U+02E8
            3 => '˧', // U+02E7
            4 => '˦', // U+02E6
            5 => '˥', // U+02E5
            _ => unreachable!(),
        }
    }

    /// Try to parse a tone letter character as a tone level.
    pub fn level_from_char(c: char) -> Option<u8> {
        match c {
            '˩' => Some(1),
            '˨' => Some(2),
            '˧' => Some(3),
            '˦' => Some(4),
            '˥' => Some(5),
            _ => None,
        }
    }
}

impl fmt::Display for Tone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &level in &self.levels {
            f.write_char(Self::tone_letter(level))?;
        }
        Ok(())
    }
}

/// Represents one of the phoneme types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phoneme {
    Consonant(Consonant),
    Vowel(Vowel),
    NonPulmonicConsonant(NonPulmonicConsonant),
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

impl From<NonPulmonicConsonant> for Phoneme {
    fn from(value: NonPulmonicConsonant) -> Self {
        Self::NonPulmonicConsonant(value)
    }
}

impl Phoneme {
    pub fn code(&self) -> char {
        match self {
            Self::Consonant(c) => c.code(),
            Self::Vowel(v) => v.code(),
            Self::NonPulmonicConsonant(c) => c.code(),
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
            .or_else(|_| NonPulmonicConsonant::try_from(value).map(Into::into))
    }
}

#[derive(Clone)]
pub struct Syllable {
    inner: SmallVec<[Segment; 8]>,
    stress: Stress,
    tone: Tone,
}

impl Syllable {
    pub fn new(seq: &[Segment]) -> Self {
        let inner = SmallVec::from(seq);
        Self {
            inner,
            stress: Stress::None,
            tone: Tone::default(),
        }
    }

    pub fn with_stress(mut self, s: Stress) -> Self {
        self.stress = s;
        self
    }

    pub fn with_tone(mut self, t: Tone) -> Self {
        self.tone = t;
        self
    }

    pub fn segments(&self) -> &[Segment] {
        self.inner.as_slice()
    }

    pub fn stress(&self) -> Stress {
        self.stress
    }

    pub fn tone(&self) -> &Tone {
        &self.tone
    }

    /// Parse an IPA string into a syllable, handling stress marks, segments with
    /// diacritics/tie bars, and trailing tone letters.
    pub fn parse_ipa(input: &str) -> Result<Self, ParseError> {
        if input.is_empty() {
            return Err(ParseError::NoInput);
        }

        let mut remaining = input;

        // Check for leading stress mark
        let stress = remaining.chars().next().and_then(Stress::from_char);
        if stress.is_some() {
            remaining = &remaining[remaining.chars().next().unwrap().len_utf8()..];
        }
        let stress = stress.unwrap_or(Stress::None);

        // Parse segments until we hit tone letters or end
        let mut segments = SmallVec::<[Segment; 8]>::new();
        while !remaining.is_empty() {
            // Check if the next char is a tone letter
            if remaining
                .chars()
                .next()
                .and_then(Tone::level_from_char)
                .is_some()
            {
                break;
            }
            let (seg, rest) = Segment::parse_ipa(remaining)?;
            segments.push(seg);
            remaining = rest;
        }

        // Collect trailing tone letters
        let mut tone_levels = SmallVec::<[u8; 3]>::new();
        for c in remaining.chars() {
            match Tone::level_from_char(c) {
                Some(level) => tone_levels.push(level),
                None => return Err(ParseError::UnknownCharacter(c)),
            }
        }
        let tone = if tone_levels.is_empty() {
            Tone::default()
        } else {
            Tone {
                levels: tone_levels,
            }
        };

        if segments.is_empty() {
            return Err(ParseError::NoInput);
        }

        Ok(Self {
            inner: segments,
            stress,
            tone,
        })
    }
}

impl PartialEq for Syllable {
    fn eq(&self, other: &Self) -> bool {
        self.stress == other.stress
            && self.tone == other.tone
            && self.segments().len() == other.segments().len()
            && self
                .segments()
                .iter()
                .zip(other.segments().iter())
                .all(|(a, b)| a == b)
    }
}

impl Eq for Syllable {}

impl fmt::Display for Syllable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.stress)?;
        for seg in self.segments() {
            write!(f, "{seg}")?;
        }
        write!(f, "{}", self.tone)?;
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
        Self::parse_ipa(s)
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    consonants: Vec<Segment>,
    vowels: Vec<Segment>,
    tones: Vec<Tone>,
}

impl Inventory {
    /// Build an inventory from bare base phones (backward-compatible path).
    /// Non-pulmonic consonants are folded into the consonants collection.
    pub fn from_base_phones(
        consonants: &[Consonant],
        vowels: &[Vowel],
        non_pulmonic_consonants: &[NonPulmonicConsonant],
    ) -> Self {
        Self {
            consonants: consonants
                .iter()
                .map(|&c| Segment::from(c))
                .chain(non_pulmonic_consonants.iter().map(|&c| Segment::from(c)))
                .collect(),
            vowels: vowels.iter().map(|&v| Segment::from(v)).collect(),
            tones: Vec::new(),
        }
    }

    /// Build an inventory with all base phones.
    pub fn with_everything() -> Self {
        Self::from_base_phones(Consonant::all(), Vowel::all(), NonPulmonicConsonant::all())
    }

    pub fn consonants(&self) -> &[Segment] {
        &self.consonants
    }

    pub fn vowels(&self) -> &[Segment] {
        &self.vowels
    }

    pub fn tones(&self) -> &[Tone] {
        &self.tones
    }

    /// Add a consonantal segment to the inventory.
    pub fn add_consonant(&mut self, seg: Segment) {
        self.consonants.push(seg);
    }

    /// Add an affricate (compound segment) to the consonant inventory.
    pub fn add_affricate(&mut self, first: Consonant, second: Consonant) {
        self.consonants.push(Segment::compound(
            Phoneme::Consonant(first),
            Phoneme::Consonant(second),
        ));
    }

    /// Add a tone to the language's tone inventory.
    pub fn add_tone(&mut self, tone: Tone) {
        self.tones.push(tone);
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.consonants.iter() {
            write!(f, "{c}")?;
        }
        f.write_char(' ')?;
        for v in self.vowels.iter() {
            write!(f, "{v}")?;
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
    fn non_pulmonics_parsing() {
        for orig in NonPulmonicConsonant::all() {
            let code = orig.code();
            let parsed = NonPulmonicConsonant::from_str(&format!("{code}")).unwrap();
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

    #[test]
    fn segment_simple_display() {
        let seg = Segment::simple(Phoneme::Consonant(Consonant::T));
        assert_eq!(format!("{seg}"), "t");
    }

    #[test]
    fn segment_with_diacritics_display() {
        let seg = Segment::with_diacritics(Phoneme::Consonant(Consonant::T), Diacritics::ASPIRATED);
        assert_eq!(format!("{seg}"), "tʰ");
    }

    #[test]
    fn segment_compound_display() {
        let seg = Segment::compound(
            Phoneme::Consonant(Consonant::T),
            Phoneme::Consonant(Consonant::Esh),
        );
        assert_eq!(format!("{seg}"), "t\u{0361}ʃ");
    }

    #[test]
    fn segment_compound_with_diacritics_display() {
        let seg = Segment::compound_with_diacritics(
            Phoneme::Consonant(Consonant::T),
            Phoneme::Consonant(Consonant::Esh),
            Diacritics::ASPIRATED,
        );
        assert_eq!(format!("{seg}"), "t\u{0361}ʃʰ");
    }

    #[test]
    fn segment_is_consonantal() {
        assert!(Segment::from(Consonant::T).is_consonantal());
        assert!(Segment::from(NonPulmonicConsonant::BilabialClick).is_consonantal());
        assert!(!Segment::from(Vowel::A).is_consonantal());
        assert!(
            Segment::compound(
                Phoneme::Consonant(Consonant::T),
                Phoneme::Consonant(Consonant::S),
            )
            .is_consonantal()
        );
    }

    #[test]
    fn segment_is_vocalic() {
        assert!(Segment::from(Vowel::A).is_vocalic());
        assert!(!Segment::from(Consonant::T).is_vocalic());
    }

    #[test]
    fn segment_manner_affricate() {
        let seg = Segment::compound(
            Phoneme::Consonant(Consonant::T),
            Phoneme::Consonant(Consonant::Esh),
        );
        assert_eq!(seg.manner(), Some(Manner::Affricate));
    }

    #[test]
    fn diacritics_multiple() {
        let d = Diacritics::ASPIRATED | Diacritics::NASALIZED;
        let seg = Segment::with_diacritics(Phoneme::Vowel(Vowel::A), d);
        let s = format!("{seg}");
        assert!(s.contains('ʰ'));
        assert!(s.contains('\u{0303}'));
    }

    #[test]
    fn stress_display() {
        assert_eq!(format!("{}", Stress::Primary), "ˈ");
        assert_eq!(format!("{}", Stress::Secondary), "ˌ");
        assert_eq!(format!("{}", Stress::None), "");
    }

    #[test]
    fn tone_display() {
        assert_eq!(format!("{}", Tone::level(5)), "˥");
        assert_eq!(format!("{}", Tone::contour(&[5, 1])), "˥˩");
        assert_eq!(format!("{}", Tone::default()), "");
    }

    #[test]
    fn syllable_with_stress_and_tone() {
        let syl = Syllable::new(&[Segment::from(Consonant::T), Segment::from(Vowel::A)])
            .with_stress(Stress::Primary)
            .with_tone(Tone::contour(&[5, 1]));
        assert_eq!(format!("{syl}"), "ˈta˥˩");
    }

    #[test]
    fn segment_parse_simple() {
        let (seg, rest) = Segment::parse_ipa("ta").unwrap();
        assert_eq!(seg, Segment::from(Consonant::T));
        assert_eq!(rest, "a");
    }

    #[test]
    fn segment_parse_with_diacritic() {
        let (seg, rest) = Segment::parse_ipa("tʰa").unwrap();
        assert_eq!(
            seg,
            Segment::with_diacritics(Phoneme::Consonant(Consonant::T), Diacritics::ASPIRATED,)
        );
        assert_eq!(rest, "a");
    }

    #[test]
    fn segment_parse_compound() {
        let input = "t\u{0361}ʃa";
        let (seg, rest) = Segment::parse_ipa(input).unwrap();
        assert_eq!(
            seg,
            Segment::compound(
                Phoneme::Consonant(Consonant::T),
                Phoneme::Consonant(Consonant::Esh),
            )
        );
        assert_eq!(rest, "a");
    }

    #[test]
    fn syllable_parse_ipa_simple() {
        let syl = Syllable::parse_ipa("ta").unwrap();
        assert_eq!(syl.segments().len(), 2);
        assert_eq!(syl.stress(), Stress::None);
        assert!(syl.tone().is_empty());
    }

    #[test]
    fn syllable_parse_ipa_with_stress() {
        let syl = Syllable::parse_ipa("ˈta").unwrap();
        assert_eq!(syl.stress(), Stress::Primary);
        assert_eq!(syl.segments().len(), 2);
    }

    #[test]
    fn syllable_parse_ipa_with_tone() {
        let syl = Syllable::parse_ipa("ta˥˩").unwrap();
        assert_eq!(syl.tone(), &Tone::contour(&[5, 1]));
        assert_eq!(syl.segments().len(), 2);
    }

    #[test]
    fn syllable_roundtrip() {
        // Build a complex syllable and verify display -> parse roundtrip
        let seg1 = Segment::compound_with_diacritics(
            Phoneme::Consonant(Consonant::T),
            Phoneme::Consonant(Consonant::Esh),
            Diacritics::ASPIRATED,
        );
        let seg2 = Segment::with_diacritics(Phoneme::Vowel(Vowel::A), Diacritics::LONG);
        let syl = Syllable::new(&[seg1, seg2])
            .with_stress(Stress::Primary)
            .with_tone(Tone::contour(&[5, 1]));
        let displayed = format!("{syl}");
        let parsed = Syllable::parse_ipa(&displayed).unwrap();
        assert_eq!(syl, parsed);
    }

    #[test]
    fn inventory_fold_non_pulmonics() {
        let inv = Inventory::from_base_phones(
            &[Consonant::T, Consonant::K],
            &[Vowel::A],
            &[NonPulmonicConsonant::BilabialClick],
        );
        // Non-pulmonics should be folded into consonants
        assert_eq!(inv.consonants().len(), 3);
        assert!(inv.consonants().iter().all(|s| s.is_consonantal()));
    }

    #[test]
    fn inventory_add_affricate() {
        let mut inv = Inventory::from_base_phones(&[Consonant::T], &[Vowel::A], &[]);
        inv.add_affricate(Consonant::T, Consonant::Esh);
        assert_eq!(inv.consonants().len(), 2);
        assert_eq!(inv.consonants()[1].manner(), Some(Manner::Affricate));
    }
}
