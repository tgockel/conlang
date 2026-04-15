//! eSpeak-ng voice driver.

use std::ffi::CString;
use std::sync::Mutex;

use super::{AudioSink, Voice};

static ESPEAK_BUFFER: Mutex<Vec<i16>> = Mutex::new(Vec::new());
static ESPEAK_INIT: Mutex<Option<u32>> = Mutex::new(None);

unsafe extern "C" fn espeak_callback(
    wav: *mut std::os::raw::c_short,
    num_samples: std::os::raw::c_int,
    _events: *mut espeakng_sys::espeak_EVENT,
) -> std::os::raw::c_int {
    if wav.is_null() || num_samples <= 0 {
        return 0;
    }
    let samples = unsafe { std::slice::from_raw_parts(wav, num_samples as usize) };
    let mut buf = ESPEAK_BUFFER.lock().unwrap();
    buf.extend_from_slice(samples);
    0
}

/// Initialize the global eSpeak-ng singleton. Safe to call multiple times;
/// initialization happens only once.
fn ensure_init() -> Result<u32, anyhow::Error> {
    let mut guard = ESPEAK_INIT.lock().unwrap();
    if let Some(rate) = *guard {
        return Ok(rate);
    }

    let sample_rate = unsafe {
        espeakng_sys::espeak_Initialize(
            espeakng_sys::espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL,
            500,
            std::ptr::null(),
            0,
        )
    };
    if sample_rate <= 0 {
        anyhow::bail!("espeak_Initialize failed (returned {sample_rate})");
    }

    unsafe {
        espeakng_sys::espeak_SetSynthCallback(Some(espeak_callback));
    }

    // Slow down from 175 wpm default to 120 wpm for clearer phoneme listening.
    unsafe {
        espeakng_sys::espeak_SetParameter(
            espeakng_sys::espeak_PARAMETER_espeakRATE,
            120,
            0,
        );
    }

    let rate = sample_rate as u32;
    *guard = Some(rate);
    Ok(rate)
}

fn set_voice(name: &str) -> Result<(), anyhow::Error> {
    let voice = CString::new(name)?;
    let err = unsafe { espeakng_sys::espeak_SetVoiceByName(voice.as_ptr()) };
    if err != espeakng_sys::espeak_ERROR_EE_OK {
        anyhow::bail!("espeak_SetVoiceByName({name}) failed (returned {err})");
    }
    Ok(())
}

fn synthesize(text: &str) -> Result<Vec<i16>, anyhow::Error> {
    {
        let mut buf = ESPEAK_BUFFER.lock().unwrap();
        buf.clear();
    }

    let c_text = CString::new(text)?;
    let text_len = c_text.as_bytes_with_nul().len();

    let err = unsafe {
        espeakng_sys::espeak_Synth(
            c_text.as_ptr() as *const std::os::raw::c_void,
            text_len,
            0,
            espeakng_sys::espeak_POSITION_TYPE_POS_CHARACTER,
            0,
            (espeakng_sys::espeakCHARS_UTF8 | espeakng_sys::espeakPHONEMES)
                as std::os::raw::c_uint,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if err != espeakng_sys::espeak_ERROR_EE_OK {
        anyhow::bail!("espeak_Synth failed (returned {err})");
    }

    let err = unsafe { espeakng_sys::espeak_Synchronize() };
    if err != espeakng_sys::espeak_ERROR_EE_OK {
        anyhow::bail!("espeak_Synchronize failed (returned {err})");
    }

    let buf = ESPEAK_BUFFER.lock().unwrap();
    Ok(buf.clone())
}

/// Convert an IPA string to eSpeak phoneme notation wrapped in `[[...]]`.
///
/// eSpeak-ng uses its own Kirshenbaum-based phoneme codes, not IPA directly.
/// This mapping covers the phonemes in the conlang inventory; unmapped characters
/// pass through as-is.
fn ipa_to_espeak_phonemes(ipa: &str) -> String {
    let mut out = String::with_capacity(ipa.len() * 2 + 4);
    out.push_str("[[");
    for ch in ipa.chars() {
        match ch {
            // Stress markers
            'ˈ' => out.push('\''),
            'ˌ' => out.push(','),

            // Syllable boundary -- skip, eSpeak handles syllabification
            '.' => {}

            // Word boundary
            ' ' => out.push(' '),

            // Consonants: direct mappings
            'p' | 'b' | 't' | 'd' | 'k' | 'g' | 'm' | 'n' | 's' | 'z' | 'f' | 'v' | 'h'
            | 'l' | 'r' | 'j' | 'w' | 'c' | 'q' => out.push(ch),

            // Consonants: mapped
            'ʈ' => out.push_str("t["),
            'ɖ' => out.push_str("d["),
            'ɟ' => out.push('J'),
            'ɢ' => out.push_str("Q\\"),
            'ʔ' => out.push('?'),
            'ɱ' => out.push('M'),
            'ɳ' => out.push_str("n."),
            'ɲ' => out.push_str("n^"),
            'ŋ' => out.push('N'),
            'ɴ' => out.push_str("n\""),
            'ʙ' => out.push_str("b<trl>"),
            'ʀ' => out.push('R'),
            'ⱱ' => out.push_str("v#"),
            'ɾ' => out.push('*'),
            'ɽ' => out.push_str("*."),
            'ɸ' => out.push('F'),
            'β' => out.push('B'),
            'θ' => out.push('T'),
            'ð' => out.push('D'),
            'ʃ' => out.push('S'),
            'ʒ' => out.push('Z'),
            'ʂ' => out.push_str("s."),
            'ʐ' => out.push_str("z."),
            'ç' => out.push('C'),
            'ʝ' => out.push_str("j\\"),
            'x' => out.push('x'),
            'ɣ' => out.push('Q'),
            'χ' => out.push('X'),
            'ʁ' => out.push_str("g\""),
            'ħ' => out.push('H'),
            'ʕ' => out.push_str("?\\"),
            'ɦ' => out.push_str("h\\"),
            'ɬ' => out.push_str("l#"),
            'ɮ' => out.push_str("l\\"),
            'ʋ' => out.push_str("v#"),
            'ɹ' => out.push_str("r\\"),
            'ɻ' => out.push_str("r\\."),
            'ɰ' => out.push_str("Q\\<apr>"),
            'ɭ' => out.push_str("l."),
            'ʎ' => out.push('L'),
            'ʟ' => out.push_str("L\\"),

            // Non-pulmonic: clicks
            'ʘ' => out.push_str("p!"),
            'ǀ' => out.push_str("t!"),
            'ǃ' => out.push_str("d!"),
            'ǂ' => out.push_str("c!"),
            'ǁ' => out.push_str("l!"),

            // Non-pulmonic: implosives
            'ɓ' => out.push_str("b`"),
            'ɗ' => out.push_str("d`"),
            'ʄ' => out.push_str("J`"),
            'ɠ' => out.push_str("g`"),
            'ʛ' => out.push_str("Q\\`"),

            // Vowels
            'i' => out.push_str("i:"),
            'y' => out.push('y'),
            'ɨ' => out.push_str("i\""),
            'ʉ' => out.push_str("u\""),
            'ɯ' => out.push_str("u-"),
            'u' => out.push_str("u:"),
            'ɪ' => out.push('I'),
            'ʏ' => out.push('Y'),
            'ʊ' => out.push('U'),
            'e' => out.push('e'),
            'ø' => out.push('W'),
            'ɘ' => out.push('@'),
            'ɵ' => out.push_str("8"),
            'ɤ' => out.push_str("o-"),
            'o' => out.push('o'),
            'ə' => out.push('@'),
            'ɛ' => out.push('E'),
            'œ' => out.push_str("W"),
            'ɜ' => out.push_str("3:"),
            'ɞ' => out.push_str("3"),
            'ʌ' => out.push('V'),
            'ɔ' => out.push_str("O:"),
            'æ' => out.push('a'),
            'ɐ' => out.push_str("a#"),
            'a' => out.push('a'),
            'ɶ' => out.push_str("a<rnd>"),
            'ɑ' => out.push_str("A:"),
            'ɒ' => out.push_str("A."),

            // Length
            'ː' => out.push(':'),
            'ˑ' => {} // half-long: no direct eSpeak equivalent, skip

            // Common diacritics (superscript modifiers)
            'ʰ' => out.push('h'),
            'ʷ' => out.push('w'),
            'ʲ' => out.push(';'),
            'ⁿ' => out.push('n'),
            'ˡ' => out.push('l'),

            // Tone markers -- skip for now
            '˥' | '˦' | '˧' | '˨' | '˩' => {}

            // Combining diacritics -- skip for trial
            '\u{0325}' | '\u{032C}' | '\u{0324}' | '\u{0330}' | '\u{0303}' | '\u{032A}'
            | '\u{033A}' | '\u{033B}' | '\u{031F}' | '\u{0320}' | '\u{0339}' | '\u{031C}'
            | '\u{031D}' | '\u{031E}' | '\u{031A}' | '\u{0329}' | '\u{032F}' | '\u{0308}'
            | '\u{0361}' => {}

            // Remaining superscript modifiers -- skip
            'ˠ' | 'ˤ' | '˞' => {}

            // Fallback: pass through
            other => out.push(other),
        }
    }
    out.push_str("]]");
    out
}

/// A voice driver backed by eSpeak-ng.
pub struct EspeakVoice {
    voice: String,
    sample_rate: u32,
}

impl EspeakVoice {
    pub fn new(voice: &str) -> Result<Self, anyhow::Error> {
        let sample_rate = ensure_init()?;
        set_voice(voice)?;
        Ok(Self {
            voice: voice.to_string(),
            sample_rate,
        })
    }
}

impl Voice for EspeakVoice {
    fn speak(&self, ipa: &str, sink: &AudioSink) -> Result<(), anyhow::Error> {
        set_voice(&self.voice)?;
        let phonemes = ipa_to_espeak_phonemes(ipa);
        let samples = synthesize(&phonemes)?;
        sink.play_pcm(&samples, self.sample_rate)
    }
}
