use anyhow::anyhow;
use clap::Parser;
use itertools::Itertools;
use std::fmt::Write;

use std::collections::HashMap;

use conlang::{generate, phone, sketch};

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
    /// Path to a JSON language sketch file.
    #[arg(long, conflicts_with_all = ["consonants", "vowels", "non_pulmonic", "pattern"])]
    pub config: Option<std::path::PathBuf>,

    #[arg(long, value_parser = parse_all::<phone::Consonant>)]
    pub consonants: Option<std::vec::Vec<phone::Consonant>>,

    #[arg(long, value_parser = parse_all::<phone::Vowel>)]
    pub vowels: Option<std::vec::Vec<phone::Vowel>>,

    #[arg(long, value_parser = parse_all::<phone::NonPulmonicConsonant>)]
    pub non_pulmonic: Option<std::vec::Vec<phone::NonPulmonicConsonant>>,

    /// A phonotological constraint pattern like `CVC` or `VV`. Specify more than once for multiple patterns.
    #[arg(long, required_unless_present = "config")]
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
            let (inventory, weights, named_sets, pattern_strings) =
                if let Some(config_path) = &cmd.config {
                    let contents = std::fs::read_to_string(config_path)?;
                    let resolved = sketch::Sketch::load(&contents)
                        .map_err(|e| anyhow!("failed to load config: {e}"))?;
                    let weights = generate::InventoryWeights {
                        consonant_weights: resolved.consonant_weights,
                        vowel_weights: resolved.vowel_weights,
                    };
                    (
                        resolved.inventory,
                        Some(weights),
                        resolved.named_sets,
                        resolved.patterns,
                    )
                } else {
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
                    (inventory, None, HashMap::new(), cmd.pattern)
                };

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

            let ctx = generate::ParseContext {
                inventory: &inventory,
                weights: weights.as_ref(),
                named_sets: &named_sets,
            };
            let patterns: Result<Vec<_>, _> = pattern_strings
                .iter()
                .map(|p| {
                    generate::WordGenerator::parse(p, &ctx)
                        .map_err(|e| format!("Could not parse pattern \"{p}\": {e}"))
                })
                .collect();
            let patterns = match patterns {
                Ok(ps) => ps,
                Err(e) => panic!("{e}"),
            };

            let mut rng = rand::rng();
            use rand::RngExt;
            for idx in rand::rng()
                .sample_iter(rand::distr::Uniform::new(0, patterns.len()).unwrap())
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
