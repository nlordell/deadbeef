mod chain;

use self::chain::Chain;
use clap::Parser;
use deadbeef_core::{Address, Contracts, Safe};
use hex::FromHexError;
use std::{num::NonZeroUsize, process, str::FromStr, sync::mpsc, thread};

/// Generate vanity addresses for Safe deployments.
#[derive(Clone, Parser)]
struct Args {
    /// The number of parallel threads to use. Defaults to the number of CPUs.
    #[arg(short = 'n', long, default_value_t = num_cpus::get())]
    threads: usize,

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

    /// Override for the fallback handler address.
    #[arg(long)]
    fallback_handler: Option<Address>,

    /// Quiet mode.
    ///
    /// Only output the transaction calldata without any extra information.
    #[arg(short, long, conflicts_with = "params")]
    quiet: bool,

    /// Parameters mode.
    ///
    /// Only output the parameters for the calling the `createProxyWithNonce`
    /// function on the `SafeProxyFactory`.
    #[arg(short = 'P', long, conflicts_with = "quiet")]
    params: bool,

    /// Override the block explorer URL for generating a link to the
    /// `SafeProxyFactory` contract. Can only be specified in parameters mode.
    #[arg(long, requires = "params")]
    explorer: Option<String>,
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

    let threads = NonZeroUsize::new(args.threads);
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
    let explorer = args.explorer.as_deref().or_else(|| args.chain.explorer());

    let setup = || {
        (
            Safe::new(contracts.clone(), args.owners.clone(), args.threshold),
            args.prefix.0.clone(),
        )
    };
    let safe = if let Some(threads) = threads {
        let (sender, receiver) = mpsc::channel();
        let _threads = (0..threads.get())
            .map(|_| {
                thread::spawn({
                    let (mut safe, prefix) = setup();
                    let result = sender.clone();
                    move || {
                        deadbeef_core::search(&mut safe, &prefix);
                        let _ = result.send(safe);
                    }
                })
            })
            .collect::<Vec<_>>();
        receiver.recv().expect("missing result")
    } else {
        let (mut safe, prefix) = setup();
        deadbeef_core::search(&mut safe, &prefix);
        safe
    };

    let transaction = safe.transaction();
    let factory = explorer
        .map(|explorer| {
            format!(
                "{}/address/{}#writeContract#F3",
                explorer, contracts.proxy_factory
            )
        })
        .unwrap_or_else(|| contracts.proxy_factory.to_string());

    if args.quiet {
        println!("0x{}", hex::encode(&transaction.calldata));
    } else if args.params {
        println!("address:     {}", safe.creation_address());
        println!("factory:     {}", factory);
        println!("singleton:   {}", contracts.singleton);
        println!("initializer: 0x{}", hex::encode(safe.initializer()));
        println!("salt nonce:  0x{}", hex::encode(safe.salt_nonce()));
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

    process::exit(0);
}
