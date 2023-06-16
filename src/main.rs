use anyhow::anyhow;
use clap::Parser;
use itertools::Itertools;
use std::fmt::Write;

use conlang::phone;

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
}

fn generate_all_syllables<'a>(
    inventory: &'a phone::Inventory,
) -> impl Iterator<Item = String> + 'a {
    let vs = inventory.vowels().iter().map(|x| format!("{}", x));
    let vvs = inventory
        .vowels()
        .iter()
        .cartesian_product(inventory.vowels().iter())
        .filter_map(|(v1, v2)| {
            if v1 != v2 {
                Some(format!("{}{}", v1, v2))
            } else {
                None
            }
        });
    let cvs = inventory
        .consonants()
        .iter()
        .cartesian_product(inventory.vowels().iter())
        .map(|(v1, v2)| format!("{}{}", v1, v2));

    vs.chain(vvs).chain(cvs)
}

fn main() {
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
            );
            for syl in generate_all_syllables(&inventory) {
                println!("{}", syl);
            }
        }
    }
}
