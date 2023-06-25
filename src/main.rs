mod address;
mod chain;
mod create2;
mod safe;

use crate::{address::Address, safe::Safe};
use chain::Chain;
use clap::Parser;
use hex::FromHexError;
use rand::{rngs::SmallRng, Rng as _, SeedableRng as _};
use safe::{Contracts, Info};
use std::{process, str::FromStr, sync::mpsc, thread};

/// Generate vanity addresses for Safe deployments.
#[derive(Clone, Parser)]
struct Args {
    /// Safe owners.
    ///
    /// Can be specified multiple times in order to specify multiple owners.
    /// They will be included in the provided order.
    #[arg(short, long = "owner", required = true, num_args = 1..)]
    owners: Vec<Address>,

    /// Owner signature threshold.
    #[arg(short, long, default_value_t = 1)]
    threshold: usize,

    /// The prefix to look for.
    #[arg(short, long)]
    prefix: Hex,

    /// The chain ID to find a vanity Safe address for. If the chain is not
    /// supported, then all of '--proxy-factory', '--proxy-init-code',
    /// '--singleton' and '--fallback-handler' must be specified.
    #[arg(short, long, default_value_t = Chain::ethereum())]
    chain: Chain,

    /// Override for the `SafeProxyFactory` address.
    #[arg(long)]
    proxy_factory: Option<Address>,

    /// Override for the `SafeProxy` init code.
    #[arg(long)]
    proxy_init_code: Option<Hex>,

    /// Override for the `Safe` singleton address.
    #[arg(long)]
    singleton: Option<Address>,

    /// Override for the Safe fallback handler address.
    #[arg(long)]
    fallback_handler: Option<Address>,

    /// Quiet mode.
    ///
    /// Only output the transaction calldata without any extra information.
    #[arg(short, long)]
    quiet: bool,
}

/// Helper type for parsing hexadecimal byte input from the command line.
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

    let contracts = args
        .chain
        .contracts()
        .map(|contracts| Contracts {
            proxy_factory: args.proxy_factory.unwrap_or(contracts.proxy_factory),
            proxy_init_code: args
                .proxy_init_code
                .clone()
                .map(|hex| hex.0)
                .unwrap_or(contracts.proxy_init_code),
            singleton: args.singleton.unwrap_or(contracts.singleton),
            fallback_handler: args.fallback_handler.unwrap_or(contracts.fallback_handler),
        })
        .or_else(|| {
            Some(Contracts {
                proxy_factory: args.proxy_factory?,
                proxy_init_code: args.proxy_init_code?.0,
                singleton: args.singleton?,
                fallback_handler: args.fallback_handler?,
            })
        })
        .expect("unsupported chain");

    let (sender, receiver) = mpsc::channel();
    let threads = (0..num_cpus::get())
        .map(|_| {
            thread::spawn({
                let safe = Safe::new(contracts.clone(), args.owners.clone(), args.threshold);
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
        println!("singleton: {}", safe.singleton);
        println!("fallback:  {}", safe.fallback_handler);
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
