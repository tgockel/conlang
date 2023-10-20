use anyhow::anyhow;
use bytes::Bytes;
use clap::Parser;
use itertools::Itertools;
use soloud::{AudioExt, LoadExt};
use std::fmt::Write;

use conlang::{gen, phone};

#[derive(Parser, Debug)]
#[command(author, version, about)]
enum Command {
    GenerateSyllables(GenerateSyllablesCmd),
}

fn parse_all<T>(src: &str) -> Result<Vec<T>, anyhow::Error>
where
    T: TryFrom<char, Error = phone::ParseError>,
{
    let mut out = Vec::with_capacity(src.len());
    let mut unknowns = Vec::new();
    for c in src.chars() {
        match T::try_from(c) {
            Ok(x) => out.push(x),
            Err(_) => unknowns.push(c),
        }
    }

    match unknowns.len() {
        0 => Ok(out),
        1 => Err(anyhow!("unknown character: {}", unknowns[0])),
        _ => {
            let mut msg = format!("unknown characters: {}", unknowns[0]);
            for c in unknowns[1..].iter() {
                write!(msg, ", {c}")?;
            }
            Err(anyhow!(msg))
        }
    }
}

#[derive(Parser, Debug)]
struct GenerateSyllablesCmd {
    #[arg(long, value_parser = parse_all::<phone::Consonant>)]
    pub consonants: Option<std::vec::Vec<phone::Consonant>>,

    #[arg(long, value_parser = parse_all::<phone::Vowel>)]
    pub vowels: Option<std::vec::Vec<phone::Vowel>>,

    #[arg(long, value_parser = parse_all::<phone::NonPulmonicConsonant>)]
    pub non_pulmonic: Option<std::vec::Vec<phone::NonPulmonicConsonant>>,

    /// A phonotological constraint pattern like `CVC` or `VV`. Specify more than once for multiple patterns.
    #[arg(long, required(true))]
    pub pattern: Vec<String>,

    /// Speak the generated phrases.
    #[arg(long)]
    pub speak: bool,
}

fn generate_all_syllables<'a>(
    inventory: &'a phone::Inventory,
) -> impl Iterator<Item = phone::Syllable> + 'a {
    let vs = inventory
        .vowels()
        .iter()
        .map(|x| phone::Syllable::new(&[(*x).into()]));
    let vvs = inventory
        .vowels()
        .iter()
        .cartesian_product(inventory.vowels().iter())
        .filter_map(|(v1, v2)| {
            if *v1 != *v2 {
                Some(phone::Syllable::new(&[(*v1).into(), (*v2).into()]))
            } else {
                None
            }
        });
    let cvs = inventory
        .consonants()
        .iter()
        .cartesian_product(inventory.vowels().iter())
        .map(|(v1, v2)| phone::Syllable::new(&[(*v1).into(), (*v2).into()]));

    vs.chain(vvs).chain(cvs)
}

struct SpeakerBox {
    polly: aws_sdk_polly::Client,
    speaker: soloud::Soloud,
}

impl SpeakerBox {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let aws_conf = aws_config::from_env().load().await;
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

#[tokio::main]
async fn main() {
    let cmd = Command::parse();
    match cmd {
        Command::GenerateSyllables(cmd) => {
            let inventory = phone::Inventory::new(
                cmd.consonants
                    .as_ref()
                    .map(|x| &x[..])
                    .unwrap_or(phone::Consonant::all()),
                cmd.vowels
                    .as_ref()
                    .map(|x| &x[..])
                    .unwrap_or(phone::Vowel::all()),
                cmd.non_pulmonic.as_ref().map(|x| &x[..]).unwrap_or(&[]),
            );

            let speaker = if cmd.speak {
                Some(SpeakerBox::new().await.unwrap())
            } else {
                None
            };

            let patterns: Result<Vec<_>, _> = cmd
                .pattern
                .iter()
                .map(|p| {
                    gen::WordGenerator::parse(&p, &inventory)
                        .map_err(|e| format!("Could not parse pattern \"{p}\": {e}"))
                })
                .collect();
            let patterns = match patterns {
                Ok(ps) => ps,
                Err(e) => panic!("{e}"),
            };

            let mut rng = rand::thread_rng();
            use rand::Rng;
            for idx in rand::thread_rng()
                .sample_iter(rand::distributions::Uniform::new(0, patterns.len()))
                .take(100)
            {
                let pattern = &patterns[idx];
                let word = pattern.generate(&mut rng);
                let ipa = word.iter().join(" ");
                println!("{}", ipa);
                if let Some(speaker) = speaker.as_ref() {
                    speaker.speak(&ipa).await.unwrap();
                }
            }
        }
    }
}
