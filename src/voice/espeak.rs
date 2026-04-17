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

/// Information about an installed eSpeak voice, returned by [`list_voices`].
pub struct EspeakVoiceInfo {
    /// Short name usable as a voice config value (e.g., "en", "mb-de5").
    pub name: String,
    /// Human-readable language or voice name.
    pub language: String,
    /// Internal eSpeak identifier path.
    pub identifier: String,
    /// For MBROLA voices, whether the required data files are installed.
    pub data_installed: bool,
}

/// Get the eSpeak-ng data directory path.
fn data_path() -> Result<std::path::PathBuf, anyhow::Error> {
    ensure_init()?;
    unsafe {
        let mut path_ptr: *const std::os::raw::c_char = std::ptr::null();
        espeakng_sys::espeak_Info(&mut path_ptr);
        if path_ptr.is_null() {
            anyhow::bail!("espeak_Info returned null data path");
        }
        let path = std::ffi::CStr::from_ptr(path_ptr)
            .to_string_lossy()
            .into_owned();
        Ok(std::path::PathBuf::from(path))
    }
}

/// Enumerate all voices known to eSpeak-ng (including MBROLA voices).
///
/// Standard voices come from `espeak_ListVoices`. MBROLA voices are discovered
/// by scanning the `voices/mb/` directory under the eSpeak data path, since the
/// API does not include them in its listing.
pub fn list_voices() -> Result<Vec<EspeakVoiceInfo>, anyhow::Error> {
    ensure_init()?;

    let mut voices = Vec::new();

    // Standard voices from the API.
    unsafe {
        let voice_list = espeakng_sys::espeak_ListVoices(std::ptr::null_mut());
        if !voice_list.is_null() {
            let mut ptr = voice_list;
            while !(*ptr).is_null() {
                let v = &**ptr;
                let name = if v.name.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(v.name)
                        .to_string_lossy()
                        .into_owned()
                };
                let identifier = if v.identifier.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(v.identifier)
                        .to_string_lossy()
                        .into_owned()
                };
                // The languages field uses a special encoding: a priority byte
                // followed by a null-terminated language code string.
                let language = if v.languages.is_null() {
                    String::new()
                } else {
                    let lang_ptr = v.languages.add(1);
                    std::ffi::CStr::from_ptr(lang_ptr)
                        .to_string_lossy()
                        .into_owned()
                };

                let short_name = identifier
                    .rsplit('/')
                    .next()
                    .unwrap_or(&identifier)
                    .to_string();

                voices.push(EspeakVoiceInfo {
                    name: short_name,
                    language: if name.is_empty() { language } else { name },
                    identifier,
                    data_installed: true,
                });

                ptr = ptr.add(1);
            }
        }
    }

    // MBROLA voices from the filesystem (not returned by espeak_ListVoices).
    if let Ok(data_dir) = data_path() {
        let mb_dir = data_dir.join("voices").join("mb");
        if let Ok(entries) = std::fs::read_dir(&mb_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                if !file_name.starts_with("mb-") {
                    continue;
                }
                let data_installed = check_mbrola_data(&file_name);
                voices.push(EspeakVoiceInfo {
                    name: file_name.clone(),
                    language: read_mbrola_language(&entry.path()),
                    identifier: format!("mb/{file_name}"),
                    data_installed,
                });
            }
        }
    }

    Ok(voices)
}

/// Read the language from an MBROLA voice definition file.
///
/// MBROLA voice files contain lines like `language de 7`. We extract the
/// language code from the first `language` directive.
fn read_mbrola_language(path: &std::path::Path) -> String {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return String::new();
    };
    for line in contents.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("language") {
            if let Some(lang) = rest.split_whitespace().next() {
                return lang.to_string();
            }
        }
    }
    String::new()
}

/// Check if MBROLA data is installed for a given voice name (e.g., "mb-de5").
fn check_mbrola_data(name: &str) -> bool {
    let data_name = name.strip_prefix("mb-").unwrap_or(name);
    let candidates = [
        format!("/usr/share/mbrola/{data_name}/{data_name}"),
        format!("/usr/share/mbrola/voices/{data_name}"),
    ];
    candidates
        .iter()
        .any(|p| std::path::Path::new(p).exists())
}

/// A voice driver backed by eSpeak-ng.
pub struct EspeakVoice {
    voice: String,
    sample_rate: u32,
    rate: Option<i32>,
    pitch: Option<i32>,
    volume: Option<i32>,
}

impl EspeakVoice {
    pub fn new(voice: &str) -> Result<Self, anyhow::Error> {
        let sample_rate = ensure_init()?;
        set_voice(voice)?;
        Ok(Self {
            voice: voice.to_string(),
            sample_rate,
            rate: Some(120),
            pitch: None,
            volume: None,
        })
    }

    pub fn from_config(config: &super::config::EspeakConfig) -> Result<Self, anyhow::Error> {
        let voice_name = config.voice.as_deref().unwrap_or("en");
        let sample_rate = ensure_init()?;
        set_voice(voice_name)?;
        Ok(Self {
            voice: voice_name.to_string(),
            sample_rate,
            rate: config.rate,
            pitch: config.pitch,
            volume: config.volume,
        })
    }

    fn apply_parameters(&self) {
        unsafe {
            if let Some(rate) = self.rate {
                espeakng_sys::espeak_SetParameter(
                    espeakng_sys::espeak_PARAMETER_espeakRATE,
                    rate,
                    0,
                );
            }
            if let Some(pitch) = self.pitch {
                espeakng_sys::espeak_SetParameter(
                    espeakng_sys::espeak_PARAMETER_espeakPITCH,
                    pitch,
                    0,
                );
            }
            if let Some(volume) = self.volume {
                espeakng_sys::espeak_SetParameter(
                    espeakng_sys::espeak_PARAMETER_espeakVOLUME,
                    volume,
                    0,
                );
            }
        }
    }
}

impl Voice for EspeakVoice {
    fn speak(&self, ipa: &str, sink: &AudioSink) -> Result<(), anyhow::Error> {
        set_voice(&self.voice)?;
        self.apply_parameters();
        let phonemes = ipa_to_espeak_phonemes(ipa);
        let samples = synthesize(&phonemes)?;
        sink.play_pcm(&samples, self.sample_rate)
    }
}
