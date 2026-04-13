use anyhow::anyhow;
use clap::Parser;
use std::fmt::Write;

use std::collections::HashMap;

use conlang::{generate, phone, sketch};
use rand::Rng;

#[cfg(feature = "pronounce")]
mod speak;
#[cfg(feature = "pronounce")]
use speak::SpeakerBox;

#[derive(Parser, Debug)]
#[command(author, version, about)]
enum Command {
    GenerateWords(GenerateWordsCmd),
    GenerateSentences(GenerateSentencesCmd),
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

fn parse_word_range(src: &str) -> Result<[u32; 2], String> {
    if let Some((min_s, max_s)) = src.split_once('-') {
        let min: u32 = min_s.parse().map_err(|e| format!("bad min: {e}"))?;
        let max: u32 = max_s.parse().map_err(|e| format!("bad max: {e}"))?;
        if min == 0 {
            return Err("minimum must be at least 1".into());
        }
        if min > max {
            return Err("minimum must not exceed maximum".into());
        }
        Ok([min, max])
    } else {
        let n: u32 = src.parse().map_err(|e| format!("bad count: {e}"))?;
        if n == 0 {
            return Err("word count must be at least 1".into());
        }
        Ok([n, n])
    }
}

/// Shared phoneme/config arguments used by both generate commands.
#[derive(Parser, Debug)]
struct PhonemeArgs {
    /// Path to a JSON language sketch file.
    #[arg(long, conflicts_with_all = ["consonants", "vowels", "non_pulmonic", "pattern"])]
    pub config: Option<std::path::PathBuf>,

    #[arg(long, value_parser = parse_all::<phone::Consonant>)]
    pub consonants: Option<std::vec::Vec<phone::Consonant>>,

    #[arg(long, value_parser = parse_all::<phone::Vowel>)]
    pub vowels: Option<std::vec::Vec<phone::Vowel>>,

    #[arg(long, value_parser = parse_all::<phone::NonPulmonicConsonant>)]
    pub non_pulmonic: Option<std::vec::Vec<phone::NonPulmonicConsonant>>,

    /// A phonotactic constraint pattern like `CVC` or `CV CVC`. Specify more than once for multiple patterns.
    #[arg(long, required_unless_present = "config")]
    pub pattern: Vec<String>,
}

struct ResolvedArgs {
    inventory: phone::Inventory,
    weights: Option<generate::InventoryWeights>,
    named_sets: HashMap<String, sketch::NamedSet>,
    sentence_config: Option<[u32; 2]>,
    word_classes: HashMap<String, sketch::ResolvedWordClass>,
    grammar: Option<Vec<String>>,
    grammar_weights: Option<Vec<u32>>,
}

impl PhonemeArgs {
    fn resolve(self) -> anyhow::Result<ResolvedArgs> {
        if let Some(config_path) = &self.config {
            let contents = std::fs::read_to_string(config_path)?;
            let resolved = sketch::Sketch::load(&contents)
                .map_err(|e| anyhow!("failed to load config: {e}"))?;
            let weights = generate::InventoryWeights {
                consonant_weights: resolved.consonant_weights,
                vowel_weights: resolved.vowel_weights,
            };
            Ok(ResolvedArgs {
                inventory: resolved.inventory,
                weights: Some(weights),
                named_sets: resolved.named_sets,
                sentence_config: resolved.sentence.map(|sc| sc.words),
                word_classes: resolved.word_classes,
                grammar: resolved.grammar,
                grammar_weights: resolved.grammar_weights,
            })
        } else {
            let inventory = phone::Inventory::from_base_phones(
                self.consonants
                    .as_ref()
                    .map(|x| &x[..])
                    .unwrap_or(phone::Consonant::all()),
                self.vowels
                    .as_ref()
                    .map(|x| &x[..])
                    .unwrap_or(phone::Vowel::all()),
                self.non_pulmonic.as_ref().map(|x| &x[..]).unwrap_or(&[]),
            );
            let weights = generate::InventoryWeights {
                consonant_weights: Some(sketch::cosine_weights(
                    inventory.consonants().len(),
                    0.0,
                )),
                vowel_weights: Some(sketch::cosine_weights(
                    inventory.vowels().len(),
                    0.0,
                )),
            };
            // Wrap CLI --pattern args into a single word class named "word".
            let mut word_classes = HashMap::new();
            word_classes.insert(
                "word".to_string(),
                sketch::ResolvedWordClass {
                    patterns: self.pattern,
                    pattern_weights: None,
                    lexicon: None,
                },
            );
            Ok(ResolvedArgs {
                inventory,
                weights: Some(weights),
                named_sets: HashMap::new(),
                sentence_config: None,
                word_classes,
                grammar: None,
                grammar_weights: None,
            })
        }
    }
}

fn parse_patterns(
    pattern_strings: &[String],
    ctx: &generate::ParseContext,
) -> Vec<generate::WordGenerator> {
    let patterns: Result<Vec<_>, _> = pattern_strings
        .iter()
        .map(|p| {
            generate::WordGenerator::parse(p, ctx)
                .map_err(|e| format!("Could not parse pattern \"{p}\": {e}"))
        })
        .collect();
    match patterns {
        Ok(ps) => ps,
        Err(e) => panic!("{e}"),
    }
}

#[derive(Parser, Debug)]
struct GenerateWordsCmd {
    #[command(flatten)]
    phonemes: PhonemeArgs,

    /// Only generate words from this word class.
    #[arg(long)]
    pub class: Option<String>,

    /// Speak the generated phrases.
    #[arg(long)]
    pub speak: bool,
}

#[derive(Parser, Debug)]
struct GenerateSentencesCmd {
    #[command(flatten)]
    phonemes: PhonemeArgs,

    /// Words per sentence as "min-max" (e.g. "3-8") or a fixed count (e.g. "5").
    #[arg(long, value_parser = parse_word_range)]
    pub words: Option<[u32; 2]>,

    /// Speak the generated phrases.
    #[arg(long)]
    pub speak: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::GenerateWords(cmd) => {
            let resolved = cmd.phonemes.resolve()?;

            if let Some(ref class_name) = cmd.class {
                if !resolved.word_classes.contains_key(class_name) {
                    let available: Vec<_> = resolved.word_classes.keys().collect();
                    anyhow::bail!(
                        "unknown word class \"{class_name}\"; available classes: {available:?}"
                    );
                }
            }

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
                inventory: &resolved.inventory,
                weights: resolved.weights.as_ref(),
                named_sets: &resolved.named_sets,
            };

            let mut rng = rand::rng();

            // Build class generators for the requested class(es).
            let class_generators: Vec<generate::ClassGenerator> = resolved
                .word_classes
                .into_iter()
                .filter(|(name, _)| {
                    cmd.class.as_ref().is_none_or(|c| c == name)
                })
                .map(|(_, wc)| {
                    let patterns = parse_patterns(&wc.patterns, &ctx);
                    let mut cg = generate::ClassGenerator::new(patterns, wc.pattern_weights);
                    if let Some(lex) = wc.lexicon {
                        cg = cg.with_lexicon(lex.size, &mut rng);
                    }
                    cg
                })
                .collect();

            for _ in 0..100 {
                let class_idx = rng.next_u64() as usize % class_generators.len();
                let (_, word) = class_generators[class_idx].generate(None, &mut rng);
                let ipa = generate::format_word(&word);
                println!("{ipa}");

                #[cfg(feature = "pronounce")]
                if let Some(speaker) = speaker.as_ref() {
                    speaker.speak(&ipa).await.unwrap();
                }
            }
            Ok(())
        }
        Command::GenerateSentences(cmd) => {
            let resolved = cmd.phonemes.resolve()?;
            let word_range = cmd
                .words
                .or(resolved.sentence_config)
                .unwrap_or([3, 8]);

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
                inventory: &resolved.inventory,
                weights: resolved.weights.as_ref(),
                named_sets: &resolved.named_sets,
            };

            let mut rng = rand::rng();

            macro_rules! output_loop {
                ($gen:expr) => {
                    for _ in 0..100 {
                        let sentence = $gen;
                        let ipa = generate::format_sentence(&sentence);
                        println!("{ipa}");

                        #[cfg(feature = "pronounce")]
                        if let Some(speaker) = speaker.as_ref() {
                            speaker.speak(&ipa).await.unwrap();
                        }
                    }
                };
            }

            // Helper: build ClassGenerator instances from resolved word classes.
            let build_classes = |word_classes: HashMap<String, sketch::ResolvedWordClass>,
                                 ctx: &generate::ParseContext,
                                 rng: &mut rand::rngs::ThreadRng| {
                let mut classes = HashMap::new();
                for (name, wc) in word_classes {
                    let patterns = parse_patterns(&wc.patterns, ctx);
                    let mut cg = generate::ClassGenerator::new(patterns, wc.pattern_weights);
                    if let Some(lex) = wc.lexicon {
                        cg = cg.with_lexicon(lex.size, rng);
                    }
                    classes.insert(name, cg);
                }
                classes
            };

            if let Some(grammar) = resolved.grammar {
                let classes = build_classes(resolved.word_classes, &ctx, &mut rng);
                let templates: Vec<Vec<String>> = grammar
                    .iter()
                    .map(|t| t.split_ascii_whitespace().map(String::from).collect())
                    .collect();
                let tsg = generate::TemplatedSentenceGenerator::new(
                    classes,
                    templates,
                    resolved.grammar_weights,
                );
                output_loop!(tsg.generate(&mut rng));
            } else {
                let classes = build_classes(resolved.word_classes, &ctx, &mut rng);
                let class_list: Vec<generate::ClassGenerator> =
                    classes.into_values().collect();
                let [min, max] = word_range;
                let sg =
                    generate::UnstructuredSentenceGenerator::new(class_list, min, max);
                output_loop!(sg.generate(&mut rng));
            }

            Ok(())
        }
    }
}
