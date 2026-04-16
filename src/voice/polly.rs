//! AWS Polly voice driver.

use super::{AudioSink, Voice};

/// A voice driver backed by AWS Polly.
pub struct PollyVoice {
    client: aws_sdk_polly::Client,
    voice_id: aws_sdk_polly::types::VoiceId,
    engine: aws_sdk_polly::types::Engine,
}

impl PollyVoice {
    pub async fn new(
        voice_id: aws_sdk_polly::types::VoiceId,
        engine: aws_sdk_polly::types::Engine,
    ) -> Result<Self, anyhow::Error> {
        let aws_conf = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = aws_sdk_polly::Client::new(&aws_conf);
        Ok(Self {
            client,
            voice_id,
            engine,
        })
    }

    pub async fn from_config(config: &super::config::PollyConfig) -> Result<Self, anyhow::Error> {
        let voice_id = aws_sdk_polly::types::VoiceId::from(
            config.voice_id.as_deref().unwrap_or("Joanna"),
        );
        let engine = aws_sdk_polly::types::Engine::from(
            config.engine.as_deref().unwrap_or("neural"),
        );
        Self::new(voice_id, engine).await
    }
}

impl Voice for PollyVoice {
    fn speak(&self, ipa: &str, sink: &AudioSink) -> Result<(), anyhow::Error> {
        let ssml = format!(r#"<phoneme alphabet="ipa" ph="{ipa}"></phoneme>"#);
        let bytes = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let resp = self
                    .client
                    .synthesize_speech()
                    .output_format(aws_sdk_polly::types::OutputFormat::OggVorbis)
                    .text_type(aws_sdk_polly::types::TextType::Ssml)
                    .text(ssml)
                    .voice_id(self.voice_id.clone())
                    .engine(self.engine.clone())
                    .send()
                    .await?;
                let blob = resp.audio_stream.collect().await?;
                Ok::<_, anyhow::Error>(blob.into_bytes())
            })
        })?;
        sink.play_ogg(&bytes)
    }
}
