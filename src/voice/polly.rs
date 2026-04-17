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

/// Information about a Polly voice, returned by [`list_voices`].
pub struct PollyVoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub language_code: String,
    pub gender: String,
    pub engines: Vec<String>,
}

/// Enumerate available Polly voices and return the configured region.
///
/// Calls the AWS DescribeVoices API, handling pagination. Fails if credentials
/// are not configured.
pub async fn list_voices() -> Result<(String, Vec<PollyVoiceInfo>), anyhow::Error> {
    let aws_conf = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let region = aws_conf
        .region()
        .map(|r| r.as_ref().to_string())
        .unwrap_or_else(|| "not set".to_string());
    let client = aws_sdk_polly::Client::new(&aws_conf);

    let mut voices = Vec::new();
    let mut next_token: Option<String> = None;
    loop {
        let mut req = client.describe_voices();
        if let Some(token) = next_token.take() {
            req = req.next_token(token);
        }
        let resp = req.send().await?;

        if let Some(voice_list) = resp.voices {
            for v in voice_list {
                let engines: Vec<String> = v
                    .supported_engines
                    .unwrap_or_default()
                    .into_iter()
                    .map(|e| e.as_str().to_string())
                    .collect();
                voices.push(PollyVoiceInfo {
                    id: v.id.map(|id| id.as_str().to_string()).unwrap_or_default(),
                    name: v.name.unwrap_or_default(),
                    language: v.language_name.unwrap_or_default(),
                    language_code: v
                        .language_code
                        .map(|lc| lc.as_str().to_string())
                        .unwrap_or_default(),
                    gender: v
                        .gender
                        .map(|g| g.as_str().to_string())
                        .unwrap_or_default(),
                    engines,
                });
            }
        }

        next_token = resp.next_token;
        if next_token.is_none() {
            break;
        }
    }

    Ok((region, voices))
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
