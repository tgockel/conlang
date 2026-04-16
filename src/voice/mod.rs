//! Voice driver abstraction and shared audio infrastructure.
//!
//! Each TTS backend (Polly, eSpeak-ng, ...) implements the [`Voice`] trait.
//! Audio playback is handled by the shared [`AudioSink`], which drivers receive
//! by reference so callers can reuse a single sink across multiple drivers.

pub mod config;
#[cfg(feature = "voice-espeak")]
pub mod espeak;
#[cfg(feature = "voice-polly")]
pub mod polly;

/// A TTS voice driver that can speak IPA text.
pub trait Voice {
    /// Speak an IPA string through the given audio sink.
    fn speak(&self, ipa: &str, sink: &AudioSink) -> Result<(), anyhow::Error>;
}

/// Shared audio output backed by `rodio`.
///
/// Owns the platform audio device and player. Both OGG/Vorbis (from Polly) and
/// raw PCM (from eSpeak-ng) can be played through the same sink.
pub struct AudioSink {
    player: rodio::Player,
    _mixer: rodio::MixerDeviceSink,
}

impl AudioSink {
    pub fn new() -> Result<Self, anyhow::Error> {
        let _mixer = rodio::DeviceSinkBuilder::open_default_sink()
            .map_err(|e| anyhow::anyhow!("Failed to open audio output: {e}"))?;
        let player = rodio::Player::connect_new(_mixer.mixer());
        Ok(Self { player, _mixer })
    }

    /// Decode and play OGG/Vorbis data (used by Polly).
    pub fn play_ogg(&self, data: &[u8]) -> Result<(), anyhow::Error> {
        let cursor = std::io::Cursor::new(data.to_vec());
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| anyhow::anyhow!("Failed to decode audio: {e}"))?;
        self.player.append(source);
        tokio::task::block_in_place(|| self.player.sleep_until_end());
        Ok(())
    }

    /// Play raw mono PCM samples at the given sample rate (used by eSpeak-ng).
    pub fn play_pcm(&self, samples: &[i16], sample_rate: u32) -> Result<(), anyhow::Error> {
        if samples.is_empty() {
            return Ok(());
        }
        let f32_samples: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();
        let channels = std::num::NonZero::new(1u16).unwrap();
        let rate = std::num::NonZero::new(sample_rate).unwrap();
        let source = rodio::buffer::SamplesBuffer::new(channels, rate, f32_samples);
        self.player.append(source);
        tokio::task::block_in_place(|| self.player.sleep_until_end());
        Ok(())
    }
}

/// Construct a default voice driver.
///
/// Prefers Polly when the `voice-polly` feature is enabled, otherwise falls
/// back to eSpeak-ng with the `mb-de5` MBROLA voice.
pub async fn create_default_driver() -> Result<Box<dyn Voice>, anyhow::Error> {
    #[cfg(feature = "voice-polly")]
    {
        return Ok(Box::new(polly::PollyVoice::new(
            aws_sdk_polly::types::VoiceId::Joanna,
            aws_sdk_polly::types::Engine::Neural,
        ).await?));
    }
    #[cfg(all(not(feature = "voice-polly"), feature = "voice-espeak"))]
    {
        return Ok(Box::new(espeak::EspeakVoice::new("mb-de5")?));
    }
}
