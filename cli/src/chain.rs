//! Module for chain-specific data.

use crate::deployment::{self, Deployment};
use deadbeef_core::Address;
use std::{
    fmt::{self, Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};

/// A Safe supported chain.
#[derive(Clone, Copy, Debug)]
pub struct Chain(u128);

impl Chain {
    /// Returns a [`Chain`] for Ethereum Mainnet.
    pub const fn ethereum() -> Self {
        Self(1)
    }

    /// Returns the [`Details`] for this chain, or [`None`] if the chain is
    /// not officially supported.
    pub fn details(&self) -> Option<Details> {
        match self.0 {
            1 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://etherscan.io"),
                singleton: Singleton::Safe,
            }),
            10 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://optimistic.etherscan.io"),
                singleton: Singleton::SafeL2,
            }),
            56 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://bscscan.com"),
                singleton: Singleton::SafeL2,
            }),
            100 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://gnosisscan.io"),
                singleton: Singleton::SafeL2,
            }),
            130 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://uniscan.xyz"),
                singleton: Singleton::SafeL2,
            }),
            137 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://polygonscan.com"),
                singleton: Singleton::SafeL2,
            }),
            146 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://sonicscan.org"),
                singleton: Singleton::SafeL2,
            }),
            196 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer {
                    url: "https://www.oklink.com/xlayer",
                    selector: "/contract#category=write&id=2",
                },
                singleton: Singleton::SafeL2,
            }),
            324 => todo!("zkSync Era is currently not supported."),
            480 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::blockscout("https://worldchain-mainnet.explorer.alchemy.com"),
                singleton: Singleton::SafeL2,
            }),
            1101 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://zkevm.polygonscan.com"),
                singleton: Singleton::SafeL2,
            }),
            5000 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://mantlescan.xyz"),
                singleton: Singleton::SafeL2,
            }),
            8453 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://basescan.org"),
                singleton: Singleton::SafeL2,
            }),
            10200 => Some(Details {
                deployment: &deployment::v1_3_0::CANONICAL,
                explorer: Explorer::blockscout("https://gnosis-chiado.blockscout.com"),
                singleton: Singleton::SafeL2,
            }),
            42161 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://arbiscan.io"),
                singleton: Singleton::SafeL2,
            }),
            42220 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::blockscout("https://explorer.celo.org/mainnet"),
                singleton: Singleton::SafeL2,
            }),
            43114 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer {
                    url: "https://snowtrace.io",
                    selector: "/contract/43114/writeContract?chainid=43114#F3",
                },
                singleton: Singleton::SafeL2,
            }),
            57073 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::blockscout("https://explorer.inkonchain.com"),
                singleton: Singleton::SafeL2,
            }),
            59144 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://lineascan.build"),
                singleton: Singleton::SafeL2,
            }),
            80094 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://berascan.com"),
                singleton: Singleton::SafeL2,
            }),
            81457 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://blastscan.io"),
                singleton: Singleton::SafeL2,
            }),
            84532 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://sepolia.basescan.org"),
                singleton: Singleton::SafeL2,
            }),
            534352 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://scrollscan.com"),
                singleton: Singleton::SafeL2,
            }),
            11155111 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::etherscan("https://sepolia.etherscan.io"),
                singleton: Singleton::SafeL2,
            }),
            1313161554 => Some(Details {
                deployment: &deployment::v1_4_1::CANONICAL,
                explorer: Explorer::blockscout("https://aurorascan.dev"),
                singleton: Singleton::SafeL2,
            }),
            _ => None,
        }
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self::ethereum()
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Chain {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "eth" => return Ok(Self(1)),
            "oeth" => return Ok(Self(10)),
            "bnb" => return Ok(Self(56)),
            "gno" => return Ok(Self(100)),
            "unichain" => return Ok(Self(130)),
            "matic" => return Ok(Self(137)),
            "sonic" => return Ok(Self(146)),
            "xlayer" => return Ok(Self(196)),
            "zksync" => return Ok(Self(324)),
            "wc" => return Ok(Self(480)),
            "zkevm" => return Ok(Self(1101)),
            "mnt" => return Ok(Self(5000)),
            "base" => return Ok(Self(8453)),
            "chiado" => return Ok(Self(10200)),
            "arb1" => return Ok(Self(42161)),
            "celo" => return Ok(Self(42220)),
            "avax" => return Ok(Self(43114)),
            "ink" => return Ok(Self(57073)),
            "linea" => return Ok(Self(59144)),
            "berachain" => return Ok(Self(80094)),
            "blast" => return Ok(Self(81457)),
            "basesep" => return Ok(Self(84532)),
            "scr" => return Ok(Self(534352)),
            "sep" => return Ok(Self(11155111)),
            "aurora" => return Ok(Self(1313161554)),
            _ => {}
        };

        let (s, radix) = match s.strip_prefix("0x") {
            Some(s) => (s, 16),
            None => (s, 10),
        };
        let value = u128::from_str_radix(s, radix)?;

        Ok(Self(value))
    }
}

/// The chain details.
pub struct Details {
    deployment: &'static Deployment,
    explorer: Explorer,
    singleton: Singleton,
}

impl Details {
    /// Returns the deployment information for the chain.
    pub fn deployment(&self) -> &Deployment {
        self.deployment
    }

    /// Returns the explorer URL to the `createProxyWithNonce` function.
    pub fn explorer(&self) -> &Explorer {
        &self.explorer
    }

    /// Returns the singleton contract kind for the chain.
    pub fn singleton(&self) -> Singleton {
        self.singleton
    }
}

/// The explorer.
pub struct Explorer {
    url: &'static str,
    selector: &'static str,
}

impl Explorer {
    /// Create a new Etherscan-like explorer instance.
    fn etherscan(url: &'static str) -> Self {
        Self {
            url,
            selector: "#writeContract#F3",
        }
    }

    /// Create a new Blockscout-like explorer instance.
    fn blockscout(url: &'static str) -> Self {
        Self {
            url,
            selector: "?tab=read_write_contract#0x1688f0b9",
        }
    }

    /// Returns the explorer URL to the `createProxyWithNonce` function.
    pub fn create_proxy_with_nonce_url(&self, proxy_factory: Address) -> String {
        format!("{}/address/{}{}", self.url, proxy_factory, self.selector)
    }
}

/// The supported singleton contract.
#[derive(Clone, Copy, Debug)]
pub enum Singleton {
    /// The `Safe` singleton contract.
    Safe,
    /// The `SafeL2` singleton contract.
    SafeL2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_proxy_with_nonce_urls() {
        for chain in [
            1, 10, 56, 100, 130, 137, 146, 196, /*324,*/ 480, 1101, 5000, 8453, 10200, 42161,
            42220, 43114, 57073, 59144, 80094, 81457, 84532, 534352, 11155111, 1313161554,
        ] {
            let details = Chain(chain).details().unwrap();
            let url = details
                .explorer
                .create_proxy_with_nonce_url(details.deployment.safe_proxy_factory.get());
            println!("{}: {}", chain, url);
        }
    }
}
