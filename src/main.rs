use anyhow::anyhow;
use clap::Parser;
use std::fmt::Write;

use std::collections::HashMap;

use conlang::{generate, phone, sketch};
use rand::RngExt;

#[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
use conlang::voice;

#[derive(Parser, Debug)]
#[command(author, version, about)]
enum Command {
    GenerateWords(GenerateWordsCmd),
    GenerateSentences(GenerateSentencesCmd),
    Config(ConfigCmd),
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
    morphology: Option<sketch::ResolvedMorphology>,
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
                morphology: resolved.morphology,
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
                consonant_weights: Some(sketch::cosine_weights(inventory.consonants().len(), 0.0)),
                vowel_weights: Some(sketch::cosine_weights(inventory.vowels().len(), 0.0)),
            };
            // Wrap CLI --pattern args into a single word class named "word".
            let mut word_classes = HashMap::new();
            word_classes.insert(
                "word".to_string(),
                sketch::ResolvedWordClass {
                    patterns: self.pattern,
                    pattern_weights: None,
                    lexicon: None,
                    stress: None,
                    secondary_stress: false,
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
                morphology: None,
            })
        }
    }
}

fn build_class_morphology(
    class_name: &str,
    morphology: &sketch::ResolvedMorphology,
    ctx: &generate::ParseContext,
) -> Option<generate::ClassMorphology> {
    let filter_affixes = |affixes: &[sketch::ResolvedAffix]| -> Vec<generate::AffixGenerator> {
        affixes
            .iter()
            .filter(|a| {
                a.applies_to
                    .as_ref()
                    .is_none_or(|classes| classes.iter().any(|c| c == class_name))
            })
            .map(|a| {
                let generator = generate::WordGenerator::parse(&a.pattern, ctx)
                    .unwrap_or_else(|e| panic!("Could not parse affix pattern {:?}: {e}", a.pattern));
                generate::AffixGenerator::new(generator, a.probability)
            })
            .collect()
    };
    let prefixes = filter_affixes(&morphology.prefixes);
    let suffixes = filter_affixes(&morphology.suffixes);
    if prefixes.is_empty() && suffixes.is_empty() {
        return None;
    }
    Some(generate::ClassMorphology::new(
        morphology.decay,
        prefixes,
        suffixes,
    ))
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

// ---------------------------------------------------------------------------
// Voice-related CLI types
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
struct VoiceArgs {
    /// Speak with a configured voice by name, or "default" for the default voice.
    #[arg(long)]
    pub speak_with: Option<String>,

    /// Speak each generated item with a random voice from the config.
    #[arg(long, conflicts_with = "speak_with")]
    pub speak: bool,

    /// Path to a voices.json configuration file.
    #[arg(long)]
    pub voice_config: Option<std::path::PathBuf>,
}

#[allow(dead_code)]
enum SpeechMode {
    Silent,
    Named(String),
    Random,
}

impl VoiceArgs {
    fn speech_mode(&self) -> SpeechMode {
        if self.speak {
            SpeechMode::Random
        } else if let Some(ref name) = self.speak_with {
            SpeechMode::Named(name.clone())
        } else {
            SpeechMode::Silent
        }
    }
}

#[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
enum Speaker {
    Single {
        sink: voice::AudioSink,
        driver: Box<dyn voice::Voice>,
    },
    Random {
        sink: voice::AudioSink,
        drivers: Vec<(String, Box<dyn voice::Voice>)>,
    },
}

#[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
impl Speaker {
    fn speak(&self, ipa: &str, rng: &mut impl rand::Rng) -> Result<(), anyhow::Error> {
        match self {
            Speaker::Single { sink, driver } => driver.speak(ipa, sink),
            Speaker::Random { sink, drivers } => {
                let idx = rng.random_range(0..drivers.len());
                drivers[idx].1.speak(ipa, sink)
            }
        }
    }
}

#[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
async fn build_speaker(voice_args: &VoiceArgs) -> anyhow::Result<Option<Speaker>> {
    match voice_args.speech_mode() {
        SpeechMode::Silent => Ok(None),
        SpeechMode::Named(ref name) => {
            let config = voice::config::load_voice_config(voice_args.voice_config.as_deref())?;
            let voice_name = if name == "default" {
                None
            } else {
                Some(name.as_str())
            };
            let entry = config.resolve(voice_name)?;
            let sink = voice::AudioSink::new()?;
            let driver = entry.create_driver().await?;
            Ok(Some(Speaker::Single { sink, driver }))
        }
        SpeechMode::Random => {
            let config = voice::config::load_voice_config(voice_args.voice_config.as_deref())?;
            if config.definitions.is_empty() {
                anyhow::bail!("--speak requires at least one voice in the config file");
            }
            let sink = voice::AudioSink::new()?;
            let drivers = config.create_all_drivers().await?;
            Ok(Some(Speaker::Random { sink, drivers }))
        }
    }
}

// ---------------------------------------------------------------------------
// Generation commands
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
struct GenerateWordsCmd {
    #[command(flatten)]
    phonemes: PhonemeArgs,

    /// Only generate words from this word class.
    #[arg(long)]
    pub class: Option<String>,

    #[command(flatten)]
    pub voice: VoiceArgs,
}

#[derive(Parser, Debug)]
struct GenerateSentencesCmd {
    #[command(flatten)]
    phonemes: PhonemeArgs,

    /// Words per sentence as "min-max" (e.g. "3-8") or a fixed count (e.g. "5").
    #[arg(long, value_parser = parse_word_range)]
    pub words: Option<[u32; 2]>,

    #[command(flatten)]
    pub voice: VoiceArgs,
}

// ---------------------------------------------------------------------------
// Config subcommand tree
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
struct ConfigCmd {
    #[command(subcommand)]
    sub: ConfigSubcommand,
}

#[derive(clap::Subcommand, Debug)]
enum ConfigSubcommand {
    Voice(VoiceConfigCmd),
}

#[derive(Parser, Debug)]
struct VoiceConfigCmd {
    #[command(subcommand)]
    sub: VoiceSubcommand,

    /// Path to a voices.json configuration file.
    #[arg(long)]
    pub voice_config: Option<std::path::PathBuf>,
}

#[derive(clap::Subcommand, Debug)]
enum VoiceSubcommand {
    /// Create a starter conlang config file with an example voices section.
    Init(VoiceInitCmd),
    /// List configured voices from the voice config file.
    List,
    /// Discover available TTS drivers and voices on this system.
    Scan,
    /// Speak a test phrase with a configured voice.
    Test(VoiceTestCmd),
}

#[derive(Parser, Debug)]
struct VoiceInitCmd {
    /// Where to write the config file.
    #[arg(long, default_value = "conlang.json")]
    pub output: std::path::PathBuf,
    /// Write to the user-level config (~/.conlang/config.json) instead of ./conlang.json.
    #[arg(long, short = 'g', conflicts_with = "output")]
    pub global: bool,
    /// Overwrite the file if it already exists.
    #[arg(long)]
    pub force: bool,
}

#[derive(Parser, Debug)]
struct VoiceTestCmd {
    /// Name of the configured voice to test.
    pub name: String,
    /// IPA text to speak. If omitted, uses a built-in test phrase.
    pub ipa: Option<String>,
}

#[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
const TEST_PHRASE: &str = "ˈpa.ta ˈka.ba ˈda.ɡa ˈsa.ʃa ˈma.na";

// ---------------------------------------------------------------------------
// Config voice handlers
// ---------------------------------------------------------------------------

/// Build a starter voices section containing one example entry per compiled-in
/// driver. Bails if no voice driver was compiled in (nothing to emit).
fn voice_config_template() -> anyhow::Result<String> {
    use serde_json::{Map, Value, json};

    #[allow(unused_mut)]
    let mut definitions = Map::new();

    #[cfg(feature = "voice-espeak")]
    definitions.insert(
        "espeak-default".to_string(),
        json!({ "driver": "espeak", "voice": "en", "rate": 150 }),
    );
    #[cfg(feature = "voice-polly")]
    definitions.insert(
        "polly-joanna".to_string(),
        json!({ "driver": "polly", "voice_id": "Joanna", "engine": "neural" }),
    );

    if definitions.is_empty() {
        anyhow::bail!(
            "no voice driver compiled in (enable the voice-polly or voice-espeak feature)"
        );
    }

    // Prefer the local, credential-free eSpeak voice as the default when present.
    let default = if cfg!(feature = "voice-espeak") {
        "espeak-default"
    } else {
        "polly-joanna"
    };

    let config = json!({
        "voices": {
            "definitions": Value::Object(definitions),
            "default": default,
        }
    });
    Ok(serde_json::to_string_pretty(&config)? + "\n")
}

fn cmd_voice_init(cmd: &VoiceInitCmd) -> anyhow::Result<()> {
    let template = voice_config_template()?;

    let output = if cmd.global {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("cannot determine home directory ($HOME not set)"))?;
        std::path::PathBuf::from(home)
            .join(".conlang")
            .join("config.json")
    } else {
        cmd.output.clone()
    };

    if output.exists() && !cmd.force {
        anyhow::bail!(
            "{} already exists; pass --force to overwrite or --output <path> \
             to write elsewhere",
            output.display()
        );
    }
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("failed to create {}: {e}", parent.display()))?;
    }
    std::fs::write(&output, template)
        .map_err(|e| anyhow::anyhow!("failed to write {}: {e}", output.display()))?;
    println!("Wrote example voice config to {}", output.display());
    println!("Edit it, then run `conlang config voice test <name>` or generate with --speak.");
    Ok(())
}

#[allow(clippy::needless_return)]
async fn cmd_voice_list(_config_path: Option<&std::path::Path>) -> anyhow::Result<()> {
    #[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
    {
        match voice::config::try_load_voice_config(_config_path)? {
            Some(config) => {
                println!("Configured voices:");
                let mut entries: Vec<_> = config.definitions.iter().collect();
                entries.sort_by_key(|(name, _)| name.as_str());
                for (name, entry) in &entries {
                    let (driver, description) = match entry {
                        voice::config::VoiceEntry::Polly(p) => (
                            "polly",
                            format!(
                                "{} ({})",
                                p.voice_id.as_deref().unwrap_or("Joanna"),
                                p.engine.as_deref().unwrap_or("neural"),
                            ),
                        ),
                        voice::config::VoiceEntry::Espeak(e) => {
                            let voice_name = e.voice.as_deref().unwrap_or("en");
                            let mut desc = voice_name.to_string();
                            if let Some(rate) = e.rate {
                                write!(desc, " (rate: {rate})")?;
                            }
                            ("espeak", desc)
                        }
                    };
                    println!("  {name:<18} {driver:<8} {description}");
                }
                if let Some(ref default) = config.default {
                    println!("\nDefault: {default}");
                }
            }
            None => {
                println!("No voice configuration found.\n");
                println!("Add a \"voices\" section to your conlang config at one of:");
                println!("  ./conlang.json");
                println!("  ~/.conlang/config.json");
                println!("\nRun `conlang config voice scan` to see available voices.");
            }
        }
        return Ok(());
    }
    #[cfg(not(any(feature = "voice-polly", feature = "voice-espeak")))]
    anyhow::bail!("no voice driver compiled in (enable the voice-polly or voice-espeak feature)");
}

#[allow(unreachable_code)]
async fn cmd_voice_scan() -> anyhow::Result<()> {
    #[cfg(not(any(feature = "voice-polly", feature = "voice-espeak")))]
    anyhow::bail!("no voice driver compiled in (enable the voice-polly or voice-espeak feature)");

    #[cfg(feature = "voice-espeak")]
    {
        let voices = voice::espeak::list_voices()?;
        println!("espeak:");

        let (mbrola, regular): (Vec<_>, Vec<_>) = voices
            .into_iter()
            .partition(|v| v.identifier.contains("mb/") || v.identifier.starts_with("mb-"));

        if !regular.is_empty() {
            println!("  Installed voices:");
            for v in &regular {
                println!("    {:<16} {}", v.name, v.language);
            }
        }

        let (mbrola_ok, mbrola_missing): (Vec<_>, Vec<_>) =
            mbrola.into_iter().partition(|v| v.data_installed);

        if !mbrola_ok.is_empty() {
            println!("  MBROLA voices (data installed):");
            for chunk in mbrola_ok.chunks(6) {
                print!("    ");
                for v in chunk {
                    print!("{:<16}", v.name);
                }
                println!();
            }
        }

        if !mbrola_missing.is_empty() {
            println!("  MBROLA voices (definition exists, data missing):");
            for chunk in mbrola_missing.chunks(6) {
                print!("    ");
                for v in chunk {
                    print!("{:<16}", v.name);
                }
                println!();
            }
        }
    }

    #[cfg(feature = "voice-polly")]
    {
        if cfg!(feature = "voice-espeak") {
            println!();
        }
        println!("polly:");
        match voice::polly::list_voices().await {
            Ok((region, voices)) => {
                println!("  AWS credentials: configured (region: {region})");
                println!("  Available voices:");
                for v in &voices {
                    let engines = v.engines.join(", ");
                    println!(
                        "    {:<16} {:<20} {:<8} {}",
                        v.id, v.language, v.gender, engines,
                    );
                }
            }
            Err(e) => {
                println!("  AWS credentials: not configured ({e})");
                println!("  Configure AWS credentials to list available voices.");
            }
        }
    }

    Ok(())
}

#[allow(clippy::needless_return)]
async fn cmd_voice_test(
    _config_path: Option<&std::path::Path>,
    _test: &VoiceTestCmd,
) -> anyhow::Result<()> {
    #[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
    {
        let config = voice::config::load_voice_config(_config_path)?;
        let entry = config.resolve(Some(&_test.name))?;
        let driver = entry.create_driver().await?;
        let sink = voice::AudioSink::new()?;
        let ipa = _test.ipa.as_deref().unwrap_or(TEST_PHRASE);
        println!("Testing voice \"{}\": {ipa}", _test.name);
        driver.speak(ipa, &sink)?;
        return Ok(());
    }
    #[cfg(not(any(feature = "voice-polly", feature = "voice-espeak")))]
    anyhow::bail!("no voice driver compiled in (enable the voice-polly or voice-espeak feature)");
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = Command::parse();
    match cmd {
        Command::GenerateWords(cmd) => {
            let resolved = cmd.phonemes.resolve()?;

            if let Some(ref class_name) = cmd.class
                && !resolved.word_classes.contains_key(class_name)
            {
                let available: Vec<_> = resolved.word_classes.keys().collect();
                anyhow::bail!(
                    "unknown word class \"{class_name}\"; available classes: {available:?}"
                );
            }

            #[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
            let speaker = build_speaker(&cmd.voice).await?;
            #[cfg(not(any(feature = "voice-polly", feature = "voice-espeak")))]
            if !matches!(cmd.voice.speech_mode(), SpeechMode::Silent) {
                anyhow::bail!(
                    "speak command specified, but no voice driver was compiled in \
                     (enable the voice-polly or voice-espeak feature)"
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
                .filter(|(name, _)| cmd.class.as_ref().is_none_or(|c| c == name))
                .map(|(name, wc)| {
                    let patterns = parse_patterns(&wc.patterns, &ctx);
                    let morph = resolved
                        .morphology
                        .as_ref()
                        .and_then(|m| build_class_morphology(&name, m, &ctx));
                    let mut cg = generate::ClassGenerator::new(
                        patterns,
                        wc.pattern_weights,
                        morph,
                        wc.stress,
                        wc.secondary_stress,
                    );
                    if let Some(lex) = wc.lexicon {
                        cg = cg.with_lexicon(lex.size, &mut rng);
                    }
                    cg
                })
                .collect();

            for _ in 0..100 {
                let class_idx = rng.random_range(0..class_generators.len());
                let (_, word) = class_generators[class_idx].generate(None, &mut rng);
                let ipa = generate::format_word(&word);
                println!("{ipa}");

                #[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
                if let Some(ref speaker) = speaker {
                    speaker.speak(&ipa, &mut rng)?;
                }
            }
            Ok(())
        }
        Command::GenerateSentences(cmd) => {
            let resolved = cmd.phonemes.resolve()?;
            let word_range = cmd.words.or(resolved.sentence_config).unwrap_or([3, 8]);

            #[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
            let speaker = build_speaker(&cmd.voice).await?;
            #[cfg(not(any(feature = "voice-polly", feature = "voice-espeak")))]
            if !matches!(cmd.voice.speech_mode(), SpeechMode::Silent) {
                anyhow::bail!(
                    "speak command specified, but no voice driver was compiled in \
                     (enable the voice-polly or voice-espeak feature)"
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

                        #[cfg(any(feature = "voice-polly", feature = "voice-espeak"))]
                        if let Some(ref speaker) = speaker {
                            speaker.speak(&ipa, &mut rng)?;
                        }
                    }
                };
            }

            // Helper: build ClassGenerator instances from resolved word classes.
            let build_classes = |word_classes: HashMap<String, sketch::ResolvedWordClass>,
                                 morphology: &Option<sketch::ResolvedMorphology>,
                                 ctx: &generate::ParseContext,
                                 rng: &mut rand::rngs::ThreadRng| {
                let mut classes = HashMap::new();
                for (name, wc) in word_classes {
                    let patterns = parse_patterns(&wc.patterns, ctx);
                    let morph = morphology
                        .as_ref()
                        .and_then(|m| build_class_morphology(&name, m, ctx));
                    let mut cg = generate::ClassGenerator::new(
                        patterns,
                        wc.pattern_weights,
                        morph,
                        wc.stress,
                        wc.secondary_stress,
                    );
                    if let Some(lex) = wc.lexicon {
                        cg = cg.with_lexicon(lex.size, rng);
                    }
                    classes.insert(name, cg);
                }
                classes
            };

            if let Some(grammar) = resolved.grammar {
                let classes =
                    build_classes(resolved.word_classes, &resolved.morphology, &ctx, &mut rng);
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
                let classes =
                    build_classes(resolved.word_classes, &resolved.morphology, &ctx, &mut rng);
                let class_list: Vec<generate::ClassGenerator> = classes.into_values().collect();
                let [min, max] = word_range;
                let sg = generate::UnstructuredSentenceGenerator::new(class_list, min, max);
                output_loop!(sg.generate(&mut rng));
            }

            Ok(())
        }
        Command::Config(config_cmd) => match config_cmd.sub {
            ConfigSubcommand::Voice(voice_cmd) => {
                let config_path = voice_cmd.voice_config.as_deref();
                match voice_cmd.sub {
                    VoiceSubcommand::Init(init) => cmd_voice_init(&init),
                    VoiceSubcommand::List => cmd_voice_list(config_path).await,
                    VoiceSubcommand::Scan => cmd_voice_scan().await,
                    VoiceSubcommand::Test(test) => cmd_voice_test(config_path, &test).await,
                }
            }
        },
    }
}
