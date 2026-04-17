//! Voice configuration parsing and loading.
//!
//! Voice definitions live in the `"voices"` section of the conlang config file.
//! The config file is searched for in:
//! 1. An explicit CLI path (`--voice-config <path>`)
//! 2. `./conlang.json` (project-local)
//! 3. `~/.conlang/config.json` (user-level)

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Voice;

/// Top-level conlang configuration file.
///
/// Each section is optional so the file can grow incrementally.
#[derive(Debug, Deserialize)]
pub struct ConlangConfig {
    #[serde(default)]
    pub voices: Option<VoiceSection>,
}

/// The `"voices"` section of the conlang config.
#[derive(Debug, Deserialize)]
pub struct VoiceSection {
    #[serde(default)]
    pub definitions: HashMap<String, VoiceEntry>,
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

impl ConlangConfig {
    /// Parse a conlang configuration from a JSON string.
    pub fn load(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl VoiceSection {
    /// Create drivers for all configured voices.
    pub async fn create_all_drivers(&self) -> Result<Vec<(String, Box<dyn Voice>)>, anyhow::Error> {
        let mut drivers = Vec::with_capacity(self.definitions.len());
        for (name, entry) in &self.definitions {
            let driver = entry.create_driver().await?;
            drivers.push((name.clone(), driver));
        }
        Ok(drivers)
    }

    /// Look up a voice entry by name, falling back to the configured default.
    pub fn resolve(&self, name: Option<&str>) -> Result<&VoiceEntry, anyhow::Error> {
        let key = match name {
            Some(n) => n,
            None => self.default.as_deref().ok_or_else(|| {
                anyhow::anyhow!("no voice name specified and no default voice configured")
            })?,
        };
        self.definitions.get(key).ok_or_else(|| {
            let mut available: Vec<_> = self.definitions.keys().map(String::as_str).collect();
            available.sort_unstable();
            anyhow::anyhow!("unknown voice \"{key}\"; available voices: {available:?}")
        })
    }
}

impl VoiceEntry {
    /// Construct a [`Voice`] driver from this configuration entry.
    #[allow(clippy::needless_return)]
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
                    return Ok(Box::new(super::espeak::EspeakVoice::from_config(_config)?));
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

/// Resolve the user home directory.
fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}

/// Load the voice section from the conlang config using the standard search order.
///
/// If `cli_path` is `Some`, only that path is tried (error if missing).
/// Otherwise searches `./conlang.json` then `~/.conlang/config.json`.
pub fn load_voice_config(cli_path: Option<&Path>) -> Result<VoiceSection, anyhow::Error> {
    try_load_voice_config(cli_path)?.ok_or_else(|| {
        anyhow::anyhow!(
            "no conlang configuration file found\n\
             \n\
             Searched:\n\
             - ./conlang.json\n\
             - ~/.conlang/config.json\n\
             \n\
             Create a conlang.json file or use --voice-config <path> to specify one."
        )
    })
}

/// Attempt to load the voice section from the conlang config, returning `None`
/// if no config file exists or the file has no `"voices"` section.
///
/// If `cli_path` is `Some` and the file doesn't exist, returns an error.
/// If `cli_path` is `None` and no candidate is found, returns `Ok(None)`.
pub fn try_load_voice_config(
    cli_path: Option<&Path>,
) -> Result<Option<VoiceSection>, anyhow::Error> {
    let config = match try_load_config(cli_path)? {
        Some(c) => c,
        None => return Ok(None),
    };
    Ok(config.voices)
}

/// Load the full conlang config using the standard search order.
fn try_load_config(cli_path: Option<&Path>) -> Result<Option<ConlangConfig>, anyhow::Error> {
    if let Some(path) = cli_path {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
        let config = ConlangConfig::load(&contents)
            .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;
        return Ok(Some(config));
    }

    let candidates: Vec<std::path::PathBuf> = [
        Some(std::path::PathBuf::from("conlang.json")),
        home_dir().map(|d| d.join(".conlang").join("config.json")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if path.exists() {
            let contents = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()))?;
            let config = ConlangConfig::load(&contents)
                .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;
            return Ok(Some(config));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_CONFIG: &str = r#"{
        "voices": {
            "definitions": {
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
        }
    }"#;

    fn voice_section(json: &str) -> VoiceSection {
        ConlangConfig::load(json).unwrap().voices.unwrap()
    }

    #[test]
    fn parse_full_config() {
        let section = voice_section(FULL_CONFIG);
        assert_eq!(section.definitions.len(), 4);
        assert_eq!(section.default.as_deref(), Some("espeak-de5"));

        match &section.definitions["polly-joanna"] {
            VoiceEntry::Polly(p) => {
                assert_eq!(p.voice_id.as_deref(), Some("Joanna"));
                assert_eq!(p.engine.as_deref(), Some("neural"));
            }
            _ => panic!("expected Polly entry"),
        }

        match &section.definitions["espeak-de5"] {
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
                "definitions": {
                    "minimal-polly": { "driver": "polly" },
                    "minimal-espeak": { "driver": "espeak" }
                }
            }
        }"#;
        let section = voice_section(json);

        match &section.definitions["minimal-polly"] {
            VoiceEntry::Polly(p) => {
                assert!(p.voice_id.is_none());
                assert!(p.engine.is_none());
            }
            _ => panic!("expected Polly entry"),
        }

        match &section.definitions["minimal-espeak"] {
            VoiceEntry::Espeak(e) => {
                assert!(e.voice.is_none());
                assert!(e.rate.is_none());
                assert!(e.pitch.is_none());
                assert!(e.volume.is_none());
            }
            _ => panic!("expected Espeak entry"),
        }

        assert!(section.default.is_none());
    }

    #[test]
    fn unknown_fields_ignored() {
        let json = r#"{
            "voices": {
                "definitions": {
                    "future-polly": {
                        "driver": "polly",
                        "voice_id": "Joanna",
                        "some_future_field": true,
                        "another_one": 42
                    }
                },
                "extra_section_field": "ignored"
            },
            "extra_top_level": "ignored"
        }"#;
        let section = voice_section(json);
        match &section.definitions["future-polly"] {
            VoiceEntry::Polly(p) => assert_eq!(p.voice_id.as_deref(), Some("Joanna")),
            _ => panic!("expected Polly entry"),
        }
    }

    #[test]
    fn unknown_driver_produces_error() {
        let json = r#"{
            "voices": {
                "definitions": {
                    "bad": { "driver": "azure-tts", "voice": "en-US" }
                }
            }
        }"#;
        let err = ConlangConfig::load(json).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("azure-tts") || msg.contains("unknown variant"),
            "error: {msg}"
        );
    }

    #[test]
    fn resolve_by_name() {
        let section = voice_section(FULL_CONFIG);
        let entry = section.resolve(Some("polly-matthew")).unwrap();
        assert!(matches!(entry, VoiceEntry::Polly(_)));
    }

    #[test]
    fn resolve_uses_default() {
        let section = voice_section(FULL_CONFIG);
        let entry = section.resolve(None).unwrap();
        assert!(matches!(entry, VoiceEntry::Espeak(_)));
    }

    #[test]
    fn resolve_unknown_name_errors() {
        let section = voice_section(FULL_CONFIG);
        let err = section.resolve(Some("nonexistent")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("nonexistent"), "error: {msg}");
        assert!(msg.contains("available voices"), "error: {msg}");
    }

    #[test]
    fn resolve_no_name_no_default_errors() {
        let json = r#"{
            "voices": {
                "definitions": { "v": { "driver": "espeak" } }
            }
        }"#;
        let section = voice_section(json);
        let err = section.resolve(None).unwrap_err();
        assert!(err.to_string().contains("no default"), "error: {err}");
    }

    #[test]
    fn no_voices_section_returns_none() {
        let json = r#"{ "some_other_section": true }"#;
        let config = ConlangConfig::load(json).unwrap();
        assert!(config.voices.is_none());
    }

    #[test]
    fn load_from_explicit_path() {
        let dir = std::env::temp_dir().join("conlang-test-voice-config");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("conlang.json");
        std::fs::write(&path, FULL_CONFIG).unwrap();

        let section = load_voice_config(Some(&path)).unwrap();
        assert_eq!(section.definitions.len(), 4);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_from_missing_explicit_path_errors() {
        let path = std::path::PathBuf::from("/tmp/conlang-nonexistent.json");
        let err = load_voice_config(Some(&path)).unwrap_err();
        assert!(err.to_string().contains("failed to read"), "error: {err}");
    }
}
