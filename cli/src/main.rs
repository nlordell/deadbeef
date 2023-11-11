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

    /// Params mode.
    ///
    /// Only output the needed fields for direct contract interaction.
    #[arg(short = 'P', long)]
    params: bool,
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
                let mut safe = Safe::new(contracts.clone(), args.owners.clone(), args.threshold);
                let prefix = args.prefix.0.clone();
                let result = sender.clone();
                move || {
                    deadbeef_core::search(&mut safe, &prefix);
                    let _ = result.send(safe);
                }
            })
        })
        .collect::<Vec<_>>();

    let safe = receiver.recv().expect("missing result");
    let transaction = safe.transaction();
    let initializer_hex = hex::encode(safe.initializer());

    if args.quiet {
        println!("0x{}", hex::encode(&transaction.calldata));
    } else if args.params {
        println!("address:      {}", safe.creation_address());
        println!("owners:       {}", args.owners[0]);
        for owner in &args.owners[1..] {
            println!("           {}", owner);
        }
        println!("--------------------------------------------------------");
        println!("_singleton:   {}", contracts.singleton);
        println!("initializer:  0x{}", initializer_hex);
        println!("saltNonce:   0x{}", hex::encode(&safe.salt_nonce()));
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
