//! Use the speaker.
//!
//! This is currently only used by the CLI.

use bytes::Bytes;
use soloud::{AudioExt, LoadExt};

pub struct SpeakerBox {
    polly: aws_sdk_polly::Client,
    speaker: soloud::Soloud,
}

impl SpeakerBox {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let aws_conf = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let polly = aws_sdk_polly::Client::new(&aws_conf);
        let speaker = soloud::Soloud::default()?;
        Ok(Self { polly, speaker })
    }

    pub async fn speak(&self, ipa: &str) -> Result<(), anyhow::Error> {
        let ogg = self.text_to_speech(ipa).await?;
        self.play_audio(&ogg).await?;
        Ok(())
    }

    async fn text_to_speech(&self, src: &str) -> Result<Bytes, anyhow::Error> {
        let resp = self
            .polly
            .synthesize_speech()
            .output_format(aws_sdk_polly::types::OutputFormat::OggVorbis)
            .text_type(aws_sdk_polly::types::TextType::Ssml)
            .text(format!(r#"<phoneme alphabet="ipa" ph="{src}"></phoneme>"#))
            .voice_id(aws_sdk_polly::types::VoiceId::Joanna)
            .engine(aws_sdk_polly::types::Engine::Neural)
            .send()
            .await?;
        let blob = resp.audio_stream.collect().await?;
        Ok(blob.into_bytes())
    }

    async fn play_audio(&self, src: &[u8]) -> Result<(), anyhow::Error> {
        let mut wav = soloud::audio::Wav::default();
        wav.load_mem(src)?;
        self.speaker.play(&wav);
        while self.speaker.voice_count() > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await
        }
        Ok(())
    }
}
