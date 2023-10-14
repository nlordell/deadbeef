mod chain;

use self::chain::Chain;
use clap::Parser;
use deadbeef_core::{Address, Contracts, Safe};
use hex::FromHexError;
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
    prefix: String,

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

// returns a vector of u8 nibbles from a hex string
// if the hex string is of odd length, the first 4 bits of each u8 will be 0
// if the hex string is of even length, each u8 will be split into two nibbles
fn hex_to_nibbles(hex_str: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    let cleaned = if hex_str.starts_with("0x") {
        &hex_str[2..]
    } else {
        hex_str
    };
    if hex_str.len() % 2 != 0 {
        cleaned.chars().map(|c| u8::from_str_radix(&c.to_string(), 16)).collect()
    } else {
        (0..cleaned.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&cleaned[i..i + 2], 16))
        .collect()
    }
}

fn main() {
    let args = Args::parse();

    let prefix = hex_to_nibbles(&args.prefix)
        .expect("Failed to decode hex string"); 

    let is_odd_length = args.prefix.len() % 2 != 0;

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
                let mut safe = Safe::new(contracts.clone(), args.owners.clone(), args.threshold);
                let prefix = prefix.clone();
                let result = sender.clone();
                move || {
                    deadbeef_core::search(&mut safe, &prefix, is_odd_length);
                    let _ = result.send(safe);
                }
            })
        })
        .collect::<Vec<_>>();

    let safe = receiver.recv().expect("missing result");
    let transaction = safe.transaction();

    if args.quiet {
        println!("0x{}", hex::encode(&transaction.calldata));
    } else {
        println!("address:   {}", safe.creation_address());
        println!("factory:   {}", contracts.proxy_factory);
        println!("singleton: {}", contracts.singleton);
        println!("fallback:  {}", contracts.fallback_handler);
        println!("owners:    {}", args.owners[0]);
        for owner in &args.owners[1..] {
            println!("           {}", owner);
        }
        println!("threshold: {}", args.threshold);
        println!("calldata:  0x{}", hex::encode(&transaction.calldata));
    }

    let _ = threads;
    process::exit(0);
}
