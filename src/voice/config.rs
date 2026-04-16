//! Voice configuration file parsing and loading.
//!
//! A `voices.json` file defines named voice entries, each specifying a TTS
//! driver and driver-specific parameters. The config file is searched for in:
//! 1. An explicit CLI path (`--voice-config <path>`)
//! 2. `./voices.json` (project-local)
//! 3. `$XDG_CONFIG_HOME/conlang/voices.json` or `~/.config/conlang/voices.json`

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Voice;

/// Top-level voice configuration file.
#[derive(Debug, Deserialize)]
pub struct VoiceConfigFile {
    pub voices: HashMap<String, VoiceEntry>,
    pub default: Option<String>,
}

/// A single voice entry, tagged by `"driver"`.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "driver")]
pub enum VoiceEntry {
    #[serde(rename = "polly")]
    Polly(PollyConfig),
    #[serde(rename = "espeak")]
    Espeak(EspeakConfig),
}

/// Driver-specific configuration for AWS Polly.
#[derive(Debug, Deserialize, Clone)]
pub struct PollyConfig {
    pub voice_id: Option<String>,
    pub engine: Option<String>,
}

/// Driver-specific configuration for eSpeak-ng.
#[derive(Debug, Deserialize, Clone)]
pub struct EspeakConfig {
    pub voice: Option<String>,
    pub rate: Option<i32>,
    pub pitch: Option<i32>,
    pub volume: Option<i32>,
}

impl VoiceConfigFile {
    /// Parse a voice configuration from a JSON string.
    pub fn load(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Look up a voice entry by name, falling back to the configured default.
    pub fn resolve(&self, name: Option<&str>) -> Result<&VoiceEntry, anyhow::Error> {
        let key = match name {
            Some(n) => n,
            None => self.default.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no voice name specified and no default voice configured")
            })?,
        };
        self.voices.get(key).ok_or_else(|| {
            let mut available: Vec<_> = self.voices.keys().map(String::as_str).collect();
            available.sort_unstable();
            anyhow::anyhow!("unknown voice \"{key}\"; available voices: {available:?}")
        })
    }
}

impl VoiceEntry {
    /// Construct a [`Voice`] driver from this configuration entry.
    pub async fn create_driver(&self) -> Result<Box<dyn Voice>, anyhow::Error> {
        match self {
            VoiceEntry::Polly(_config) => {
                #[cfg(feature = "voice-polly")]
                {
                    return Ok(Box::new(
                        super::polly::PollyVoice::from_config(_config).await?,
                    ));
                }
                #[cfg(not(feature = "voice-polly"))]
                {
                    anyhow::bail!(
                        "this voice uses the Polly driver, but Polly support was not compiled in \
                         (rebuild with --features voice-polly)"
                    );
                }
            }
            VoiceEntry::Espeak(_config) => {
                #[cfg(feature = "voice-espeak")]
                {
                    return Ok(Box::new(
                        super::espeak::EspeakVoice::from_config(_config)?,
                    ));
                }
                #[cfg(not(feature = "voice-espeak"))]
                {
                    anyhow::bail!(
                        "this voice uses the eSpeak driver, but eSpeak support was not compiled \
                         in (rebuild with --features voice-espeak)"
                    );
                }
            }
        }
    }
}

/// Resolve the user-level config directory (`$XDG_CONFIG_HOME` or `~/.config`).
fn user_config_dir() -> Option<std::path::PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Some(std::path::PathBuf::from(xdg))
    } else if let Ok(home) = std::env::var("HOME") {
        Some(std::path::PathBuf::from(home).join(".config"))
    } else {
        None
    }
}

/// Load a voice configuration file using the standard search order.
///
/// If `cli_path` is `Some`, only that path is tried (error if missing).
/// Otherwise searches `./voices.json` then `~/.config/conlang/voices.json`.
pub fn load_voice_config(cli_path: Option<&Path>) -> Result<VoiceConfigFile, anyhow::Error> {
    if let Some(path) = cli_path {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
        return VoiceConfigFile::load(&contents)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()));
    }

    let candidates: Vec<std::path::PathBuf> = [
        Some(std::path::PathBuf::from("voices.json")),
        user_config_dir().map(|d| d.join("conlang").join("voices.json")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if path.exists() {
            let contents = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
            let config = VoiceConfigFile::load(&contents)
                .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;
            return Ok(config);
        }
    }

    anyhow::bail!(
        "no voice configuration file found\n\
         \n\
         Searched:\n\
         - ./voices.json\n\
         - ~/.config/conlang/voices.json\n\
         \n\
         Create a voices.json file or use --voice-config <path> to specify one."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_CONFIG: &str = r#"{
        "voices": {
            "polly-joanna": {
                "driver": "polly",
                "voice_id": "Joanna",
                "engine": "neural"
            },
            "polly-matthew": {
                "driver": "polly",
                "voice_id": "Matthew",
                "engine": "standard"
            },
            "espeak-de5": {
                "driver": "espeak",
                "voice": "mb-de5",
                "rate": 120,
                "pitch": 50,
                "volume": 100
            },
            "espeak-default": {
                "driver": "espeak",
                "voice": "en",
                "rate": 150
            }
        },
        "default": "espeak-de5"
    }"#;

    #[test]
    fn parse_full_config() {
        let config = VoiceConfigFile::load(FULL_CONFIG).unwrap();
        assert_eq!(config.voices.len(), 4);
        assert_eq!(config.default.as_deref(), Some("espeak-de5"));

        match &config.voices["polly-joanna"] {
            VoiceEntry::Polly(p) => {
                assert_eq!(p.voice_id.as_deref(), Some("Joanna"));
                assert_eq!(p.engine.as_deref(), Some("neural"));
            }
            _ => panic!("expected Polly entry"),
        }

        match &config.voices["espeak-de5"] {
            VoiceEntry::Espeak(e) => {
                assert_eq!(e.voice.as_deref(), Some("mb-de5"));
                assert_eq!(e.rate, Some(120));
                assert_eq!(e.pitch, Some(50));
                assert_eq!(e.volume, Some(100));
            }
            _ => panic!("expected Espeak entry"),
        }
    }

    #[test]
    fn missing_optional_fields() {
        let json = r#"{
            "voices": {
                "minimal-polly": { "driver": "polly" },
                "minimal-espeak": { "driver": "espeak" }
            }
        }"#;
        let config = VoiceConfigFile::load(json).unwrap();

        match &config.voices["minimal-polly"] {
            VoiceEntry::Polly(p) => {
                assert!(p.voice_id.is_none());
                assert!(p.engine.is_none());
            }
            _ => panic!("expected Polly entry"),
        }

        match &config.voices["minimal-espeak"] {
            VoiceEntry::Espeak(e) => {
                assert!(e.voice.is_none());
                assert!(e.rate.is_none());
                assert!(e.pitch.is_none());
                assert!(e.volume.is_none());
            }
            _ => panic!("expected Espeak entry"),
        }

        assert!(config.default.is_none());
    }

    #[test]
    fn unknown_fields_ignored() {
        let json = r#"{
            "voices": {
                "future-polly": {
                    "driver": "polly",
                    "voice_id": "Joanna",
                    "some_future_field": true,
                    "another_one": 42
                }
            },
            "extra_top_level": "ignored"
        }"#;
        let config = VoiceConfigFile::load(json).unwrap();
        match &config.voices["future-polly"] {
            VoiceEntry::Polly(p) => assert_eq!(p.voice_id.as_deref(), Some("Joanna")),
            _ => panic!("expected Polly entry"),
        }
    }

    #[test]
    fn unknown_driver_produces_error() {
        let json = r#"{
            "voices": {
                "bad": { "driver": "azure-tts", "voice": "en-US" }
            }
        }"#;
        let err = VoiceConfigFile::load(json).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("azure-tts") || msg.contains("unknown variant"), "error: {msg}");
    }

    #[test]
    fn resolve_by_name() {
        let config = VoiceConfigFile::load(FULL_CONFIG).unwrap();
        let entry = config.resolve(Some("polly-matthew")).unwrap();
        assert!(matches!(entry, VoiceEntry::Polly(_)));
    }

    #[test]
    fn resolve_uses_default() {
        let config = VoiceConfigFile::load(FULL_CONFIG).unwrap();
        let entry = config.resolve(None).unwrap();
        assert!(matches!(entry, VoiceEntry::Espeak(_)));
    }

    #[test]
    fn resolve_unknown_name_errors() {
        let config = VoiceConfigFile::load(FULL_CONFIG).unwrap();
        let err = config.resolve(Some("nonexistent")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("nonexistent"), "error: {msg}");
        assert!(msg.contains("available voices"), "error: {msg}");
    }

    #[test]
    fn resolve_no_name_no_default_errors() {
        let json = r#"{ "voices": { "v": { "driver": "espeak" } } }"#;
        let config = VoiceConfigFile::load(json).unwrap();
        let err = config.resolve(None).unwrap_err();
        assert!(err.to_string().contains("no default"), "error: {err}");
    }

    #[test]
    fn load_from_explicit_path() {
        let dir = std::env::temp_dir().join("conlang-test-voice-config");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test-voices.json");
        std::fs::write(&path, FULL_CONFIG).unwrap();

        let config = load_voice_config(Some(&path)).unwrap();
        assert_eq!(config.voices.len(), 4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_from_missing_explicit_path_errors() {
        let path = std::path::PathBuf::from("/tmp/conlang-nonexistent-voices.json");
        let err = load_voice_config(Some(&path)).unwrap_err();
        assert!(err.to_string().contains("failed to read"), "error: {err}");
    }
}
