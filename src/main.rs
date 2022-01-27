mod address;
mod create2;
mod safe;

use crate::{address::Address, safe::Safe};
use clap::Parser;
use hex::FromHexError;
use rand::{rngs::SmallRng, Rng as _, SeedableRng as _};
use safe::Info;
use std::{process, str::FromStr, sync::mpsc, thread};

/// Generate vanity addresses for Gnosis Safe deployments.
#[derive(Clone, Parser)]
struct Args {
    /// Safe owners.
    ///
    /// Can be specified multiple times in order to specify multiple owners.
    /// They will be included in the provided order.
    #[clap(short, long = "owner", required = true, min_values = 1)]
    owners: Vec<Address>,

    /// Owner signature threshold.
    #[clap(short, long, default_value_t = 1)]
    threshold: usize,

    /// The prefix to look for.
    #[clap(short, long)]
    prefix: Hex,

    /// Quiet mode.
    ///
    /// Only output the transaction calldata without any extra inforamtion.
    #[clap(short, long)]
    quiet: bool,
}

/// Helper type for parsing hexadecimal input from the command line.
#[derive(Clone)]
struct Hex(Vec<u8>);

impl FromStr for Hex {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s.strip_prefix("0x").unwrap_or(s)).map(Hex)
    }
}

fn main() {
    let args = Args::parse();

    let (sender, receiver) = mpsc::channel();
    let threads = (0..num_cpus::get())
        .map(|_| {
            thread::spawn({
                let safe = Safe::new(args.owners.clone(), args.threshold);
                let prefix = args.prefix.0.clone();
                let result = sender.clone();
                move || search_vanity_safe(safe, &prefix, result)
            })
        })
        .collect::<Vec<_>>();

    let safe = receiver.recv().expect("missing result");

    if args.quiet {
        println!("0x{}", hex::encode(&safe.calldata));
    } else {
        println!("address:   {}", safe.creation_address);
        println!("factory:   {}", safe.factory);
        println!("singleton: {}", safe.factory);
        println!("fallback:  {}", safe.factory);
        println!("owners:    {}", safe.owners[0]);
        for owner in &safe.owners[1..] {
            println!("           {}", owner);
        }
        println!("threshold: {}", safe.threshold);
        println!("calldata:  0x{}", hex::encode(&safe.calldata));
    }

    let _ = threads;
    process::exit(0);
}

fn search_vanity_safe(mut safe: Safe, prefix: &[u8], result: mpsc::Sender<Info>) {
    let mut rng = SmallRng::from_entropy();

    while !safe.creation_address().0.starts_with(prefix) {
        safe.update_salt_nonce(|n| rng.fill(n));
    }

    let _ = result.send(safe.info());
}
