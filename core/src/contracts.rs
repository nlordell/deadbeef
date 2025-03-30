use crate::Address;

/// Safe contract data on a given chain.
#[derive(Clone)]
pub struct Contracts {
    /// The address of the `SafeProxyFactory` contract.
    pub proxy_factory: Address,
    /// The `SafeProxy` init code.
    pub proxy_init_code: Vec<u8>,
    /// The `Safe` singleton address.
    pub singleton: Address,
    /// The optional `SafeToL2Setup` setup to use.
    pub setup: Option<SafeToL2Setup>,
    /// The fallback handler address to use (for example, the
    /// `CompatibilityFallbackHandler`).
    pub fallback_handler: Address,
}

/// Safe multi-chain setup.
#[derive(Clone)]
pub struct SafeToL2Setup {
    /// The addres of the setup contract.
    pub address: Address,
    /// The `SafeL2` singleton for the setup.
    pub singleton_l2: Address,
}
