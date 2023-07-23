//! # Phonetics
//!
//! Phonetics form the basis of spoken language. This module contains `Phoneme`s as the basic building block of the
//! language and `Syllable`s to tie them together.

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

impl TryFrom<char> for Place {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        todo!()
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
}

impl TryFrom<char> for Manner {
    type Error = ParseError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        todo!()
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

/// For a `Vowel`, how high the tongue is in the mouth. The value has a range 0 to 10, inclusive.
///
/// The International Phonetic Alphabet classifies vowel sounds into "Close," "Close-Mid," "Open-Mid," and "Open." An
/// `enum` for this is not sufficient, since characters like "near-close near-front unrounded vowel" (ɪ) and "near-open
/// central vowel" (ɐ) land in between these categories. Values of 0 represent fully-open vowels like a and ɒ; values of
/// 10 represent the fully-closed vowels like i and ɯ; and other values somewhere in between.
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
    inner: smallvec::SmallVec<[Phoneme; 8]>,
}

impl Syllable {
    pub fn new(seq: &[Phoneme]) -> Self {
        let inner = smallvec::SmallVec::from(seq);
        Self { inner }
    }

    pub fn parts(&self) -> &[Phoneme] {
        self.inner.as_slice()
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
    non_pulmonic_consonants: Vec<NonPulmonicConsonant>,
}

impl Inventory {
    pub fn new(
        consonants: impl Into<Vec<Consonant>>,
        vowels: impl Into<Vec<Vowel>>,
        non_pulmonic_consonants: impl Into<Vec<NonPulmonicConsonant>>,
    ) -> Self {
        Self {
            consonants: consonants.into(),
            vowels: vowels.into(),
            non_pulmonic_consonants: non_pulmonic_consonants.into(),
        }
    }

    pub fn with_everything() -> Self {
        Self::new(Consonant::all(), Vowel::all(), NonPulmonicConsonant::all())
    }

    pub fn consonants(&self) -> &[Consonant] {
        &self.consonants
    }

    pub fn vowels(&self) -> &[Vowel] {
        &self.vowels
    }

    pub fn non_pulmonic_consonants(&self) -> &[NonPulmonicConsonant] {
        &self.non_pulmonic_consonants
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
}
