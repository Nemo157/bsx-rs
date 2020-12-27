use anyhow::anyhow;
use std::{
    io::{self, Read, Write},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Debug)]
enum Alphabet {
    Bitcoin,
    Monero,
    Ripple,
    Flickr,
    Custom(bsx::DynamicAlphabet<Vec<u8>>),
}

impl Alphabet {
    fn as_alphabet(&self) -> &dyn bsx::Alphabet {
        match self {
            Alphabet::Bitcoin => bsx::Alphabet::BITCOIN,
            Alphabet::Monero => bsx::Alphabet::MONERO,
            Alphabet::Ripple => bsx::Alphabet::RIPPLE,
            Alphabet::Flickr => bsx::Alphabet::FLICKR,
            Alphabet::Custom(custom) => custom,
        }
    }
}

impl FromStr for Alphabet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bitcoin" => Alphabet::Bitcoin,
            "monero" => Alphabet::Monero,
            "ripple" => Alphabet::Ripple,
            "flickr" => Alphabet::Flickr,
            custom if custom.starts_with("custom(") && custom.ends_with(')') => {
                let alpha = custom.trim_start_matches("custom(").trim_end_matches(')');
                Alphabet::Custom(bsx::DynamicAlphabet::new(alpha.into())?)
            }
            other => {
                return Err(anyhow!("'{}' is not a known alphabet", other));
            }
        })
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "bsx", setting = structopt::clap::AppSettings::ColoredHelp)]
/// A utility for encoding/decoding arbitrary base encoded data.
struct Args {
    /// Decode input
    #[structopt(long, short = "d")]
    decode: bool,

    /// Which alphabet to decode/encode with [possible values: bitcoin, monero,
    /// ripple, flickr or custom(abc...xyz)]
    #[structopt(long, short = "a", default_value = "bitcoin")]
    alphabet: Alphabet,
}

const INITIAL_INPUT_CAPACITY: usize = 4096;

fn main() -> anyhow::Result<()> {
    let args = Args::from_iter_safe(std::env::args_os())?;

    if args.decode {
        let mut input = String::with_capacity(INITIAL_INPUT_CAPACITY);
        io::stdin().read_to_string(&mut input)?;
        let trimmed = input.trim_end();
        let output = bsx::decode(trimmed)
            .with_alphabet(args.alphabet.as_alphabet())
            .into_vec()?;
        io::stdout().write_all(&output)?;
    } else {
        let mut input = Vec::with_capacity(INITIAL_INPUT_CAPACITY);
        io::stdin().read_to_end(&mut input)?;
        let output = bsx::encode(input)
            .with_alphabet(args.alphabet.as_alphabet())
            .into_string();
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}
