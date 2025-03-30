mod chain;
mod deployment;

use self::chain::{Chain, Singleton};
use chain::Details;
use clap::Parser;
use deadbeef_core::{config, Address, Configuration, NonZeroAddress, Safe};
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
    owners: Vec<NonZeroAddress>,

    /// Owner signature threshold.
    #[arg(short, long, default_value_t = 1)]
    threshold: usize,

    /// The prefix to look for.
    #[arg(short, long)]
    prefix: Hex,

    /// The chain ID to find a vanity Safe address for. If the chain is not
    /// supported, then all of '--proxy-factory', '--proxy-init-code', and
    /// '--singleton' must be specified.
    #[arg(short, long, default_value_t = Chain::ethereum())]
    chain: Chain,

    /// Override for the `SafeProxyFactory` address.
    #[arg(long)]
    proxy_factory: Option<NonZeroAddress>,

    /// Override for the `SafeProxy` init code.
    #[arg(long)]
    proxy_init_code: Option<Hex>,

    /// Override for the `Safe` singleton address.
    #[arg(long)]
    singleton: Option<NonZeroAddress>,

    /// Override for the `SafeL2` singleton address.
    ///
    /// For unsupported chains, if this is specified then `--safe-to-l2-setup`
    /// must also be specified.
    #[arg(long)]
    l2_singleton: Option<NonZeroAddress>,

    /// Override for the `SafeToL2Setup` address.
    ///
    /// Specifying the 0 address (or not specifying the contract address for
    /// unknown chains) will disable this feature (which is not recommended).
    ///
    /// For unsupported chains, if this is specified then `--l2-singleton` must
    /// also be specified.
    #[arg(long)]
    safe_to_l2_setup: Option<Address>,

    /// Override for the fallback handler address.
    #[arg(long)]
    fallback_handler: Option<NonZeroAddress>,

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
}

/// Helper type for parsing hexadecimal byte input from the command line.
#[derive(Clone)]
struct Hex(Vec<u8>);

impl Hex {
    fn cloned(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl FromStr for Hex {
    type Err = FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s.strip_prefix("0x").unwrap_or(s)).map(Hex)
    }
}

fn main() {
    let args = Args::parse();

    let threads = NonZeroUsize::new(args.threads);
    let chain = args.chain.details();
    let config = chain
        .as_ref()
        .map(|details| {
            let contracts = details.deployment();
            let setup = args
                .safe_to_l2_setup
                .unwrap_or(contracts.safe_to_l2_setup)
                .non_zero()
                .map(|address| config::SafeToL2Setup {
                    address,
                    l2_singleton: args.l2_singleton.unwrap_or(contracts.safe_l2),
                });

            Configuration {
                proxy: config::Proxy {
                    factory: args.proxy_factory.unwrap_or(contracts.safe_proxy_factory),
                    init_code: args
                        .proxy_init_code
                        .as_ref()
                        .map(Hex::cloned)
                        .unwrap_or(contracts.safe_proxy_init_code.to_vec()),
                    singleton: args.singleton.unwrap_or_else(|| {
                        match (&setup, details.singleton()) {
                            // If we are using the `SafeToL2Setup`, then always
                            // use the `Safe` singleton.
                            (Some(_), _) => contracts.safe,
                            (_, Singleton::Safe) => contracts.safe,
                            (_, Singleton::SafeL2) => contracts.safe_l2,
                        }
                    }),
                },
                account: config::Account {
                    owners: args.owners.clone(),
                    threshold: args.threshold,
                    setup,
                    fallback_handler: args
                        .fallback_handler
                        .or_else(|| contracts.compatibility_fallback_handler.non_zero()),
                    identifier: None,
                },
            }
        })
        .or_else(|| {
            Some(Configuration {
                proxy: config::Proxy {
                    factory: args.proxy_factory?,
                    init_code: args.proxy_init_code?.cloned(),
                    singleton: args.singleton?,
                },
                account: config::Account {
                    owners: args.owners.clone(),
                    threshold: args.threshold,
                    setup: match (
                        args.safe_to_l2_setup.and_then(Address::non_zero),
                        args.l2_singleton,
                    ) {
                        (None, None) => None,
                        // For unsupported chains, if either `SafeToL2Setup` or
                        // `SafeL2` is specified, then both must be specified.
                        (safe_to_l2_setup, l2_singleton) => Some(config::SafeToL2Setup {
                            address: safe_to_l2_setup?,
                            l2_singleton: l2_singleton?,
                        }),
                    },
                    fallback_handler: args.fallback_handler,
                    identifier: None,
                },
            })
        })
        .expect("unsupported chain");
    let explorer = chain.as_ref().map(Details::explorer);

    let setup = || (Safe::new(config.clone()), args.prefix.0.clone());
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

    if args.quiet {
        println!("0x{}", hex::encode(&transaction.calldata));
    } else if args.params {
        let factory = explorer
            .map(|explorer| explorer.create_proxy_with_nonce_url(config.proxy.factory.get()))
            .unwrap_or_else(|| config.proxy.factory.to_string());

        println!("address:     {}", safe.creation_address());
        println!("factory:     {}", factory);
        println!("singleton:   {}", config.proxy.singleton);
        println!("initializer: 0x{}", hex::encode(safe.initializer()));
        println!("salt nonce:  0x{}", hex::encode(safe.salt_nonce()));
    } else {
        let (to, data) = config
            .account
            .setup
            .as_ref()
            .map(|setup| (setup.address.get(), setup.encode()))
            .unwrap_or_default();
        let fallback = config
            .account
            .fallback_handler
            .map(NonZeroAddress::get)
            .unwrap_or_default();

        println!("address:     {}", safe.creation_address());
        println!("factory:     {}", config.proxy.factory);
        println!("singleton:   {}", config.proxy.singleton);
        println!("initializer: 0x{}", hex::encode(safe.initializer()));
        println!("salt nonce:  0x{}", hex::encode(safe.salt_nonce()));
        println!("---");
        println!("owners:      {}", config.account.owners[0]);
        for owner in &args.owners[1..] {
            println!("             {}", owner);
        }
        println!("threshold:   {}", config.account.threshold);
        println!("to:          {}", to);
        println!("data:        0x{}", hex::encode(&data));
        println!("fallback:    {}", fallback);
        println!("---");
        println!("calldata:    0x{}", hex::encode(&transaction.calldata));
    }

    process::exit(0);
}
