//! Use the speaker.
//!
//! This is currently only used by the CLI.

use bytes::Bytes;

pub struct SpeakerBox {
    polly: aws_sdk_polly::Client,
    sink: rodio::Sink,
    _stream: rodio::OutputStream,
}

impl SpeakerBox {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let aws_conf = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let polly = aws_sdk_polly::Client::new(&aws_conf);
        let (_stream, handle) = rodio::OutputStream::try_default()
            .map_err(|e| anyhow::anyhow!("Failed to open audio output: {e}"))?;
        let sink = rodio::Sink::try_new(&handle)
            .map_err(|e| anyhow::anyhow!("Failed to create audio sink: {e}"))?;
        Ok(Self { polly, sink, _stream })
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
        let cursor = std::io::Cursor::new(src.to_vec());
        let source = rodio::Decoder::new(cursor)
            .map_err(|e| anyhow::anyhow!("Failed to decode audio: {e}"))?;
        self.sink.append(source);
        tokio::task::block_in_place(|| self.sink.sleep_until_end());
        Ok(())
    }
}
