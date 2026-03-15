use anyhow::anyhow;
use clap::Parser;
use itertools::Itertools;
use std::fmt::Write;

use conlang::{generate, phone};

#[cfg(feature = "pronounce")]
mod speak;
#[cfg(feature = "pronounce")]
use speak::SpeakerBox;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::GenerateSyllables(cmd) => {
            let inventory = phone::Inventory::from_base_phones(
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

            #[cfg(feature = "pronounce")]
            let speaker = if cmd.speak {
                Some(SpeakerBox::new().await.unwrap())
            } else {
                None
            };
            #[cfg(not(feature = "pronounce"))]
            if cmd.speak {
                anyhow::bail!(
                    "speak command specified, but this has not been compiled with `pronounce`"
                );
            }

            let patterns: Result<Vec<_>, _> = cmd
                .pattern
                .iter()
                .map(|p| {
                    generate::WordGenerator::parse(p, &inventory)
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

                #[cfg(feature = "pronounce")]
                if let Some(speaker) = speaker.as_ref() {
                    speaker.speak(&ipa).await.unwrap();
                }
            }
            Ok(())
        }
    }
}
