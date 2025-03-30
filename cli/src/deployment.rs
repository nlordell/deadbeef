use deadbeef_core::{address, hex, Address, NonZeroAddress};

/// Safe contract deployment.
#[derive(Clone)]
pub struct Deployment {
    /// The `SafeProxyFactory` contract address.
    pub safe_proxy_factory: NonZeroAddress,
    /// The `SafeProxy` init code.
    pub safe_proxy_init_code: &'static [u8],
    /// The `Safe` singleton address.
    pub safe: NonZeroAddress,
    /// The `SafeL2` singleton address.
    pub safe_l2: NonZeroAddress,
    /// The `SafeToL2Setup` setup contract address.
    pub safe_to_l2_setup: Address,
    /// The `CompatibilityFallbackHandler` default fallback handler address.
    pub compatibility_fallback_handler: Address,
}

/// The Safe v1.4.1 contract deployments.
///
/// Addresses can be found in the Safe deployments repository:
/// <https://github.com/safe-global/safe-deployments/tree/main/src/assets/v1.4.1>
/// The `proxyCreationCode` can be read from the from the proxy factory:
/// <https://etherscan.io/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67#readContract#F2>
pub mod v1_4_1 {
    use super::*;

    /// The canonical contract deployment.
    pub static CANONICAL: Deployment = Deployment {
        safe_proxy_factory: address!(nz "4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67"),
        safe_proxy_init_code: &hex!(
            "608060405234801561001057600080fd5b506040516101e63803806101e68339
             818101604052602081101561003357600080fd5b810190808051906020019092
             9190505050600073ffffffffffffffffffffffffffffffffffffffff168173ff
             ffffffffffffffffffffffffffffffffffffff1614156100ca576040517f08c3
             79a0000000000000000000000000000000000000000000000000000000008152
             6004018080602001828103825260228152602001806101c46022913960400191
             505060405180910390fd5b806000806101000a81548173ffffffffffffffffff
             ffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffff
             ffffffffff1602179055505060ab806101196000396000f3fe608060405273ff
             ffffffffffffffffffffffffffffffffffffff600054167fa619486e00000000
             0000000000000000000000000000000000000000000000006000351415605057
             8060005260206000f35b3660008037600080366000845af43d6000803e600081
             14156070573d6000fd5b3d6000f3fea264697066735822122003d1488ee65e08
             fa41e58e888a9865554c535f2c77126a82cb4c0f917f31441364736f6c634300
             07060033496e76616c69642073696e676c65746f6e2061646472657373207072
             6f7669646564"
        ),
        safe: address!(nz "41675C099F32341bf84BFc5382aF534df5C7461a"),
        safe_l2: address!(nz "29fcB43b46531BcA003ddC8FCB67FFE91900C762"),
        safe_to_l2_setup: address!("BD89A1CE4DDe368FFAB0eC35506eEcE0b1fFdc54"),
        compatibility_fallback_handler: address!("fd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99"),
    };

    #[cfg(test)]
    mod tests {
        use super::*;
        use deadbeef_core::{config, Configuration, Safe};

        #[test]
        fn proxy_init_code_digest() {
            let proxy = config::Proxy {
                factory: CANONICAL.safe_proxy_factory,
                init_code: CANONICAL.safe_proxy_init_code.to_vec(),
                singleton: CANONICAL.safe,
            };
            assert_eq!(
                proxy.init_code_hash(),
                hex!("76733d705f71b79841c0ee960a0ca880f779cde7ef446c989e6d23efc0a4adfb"),
            );
        }

        #[test]
        fn compute_address() {
            // <https://etherscan.io/tx/0x764675dc513abc36844acf38ac0ef783b0c3e900f8a7d4695bff734e1b0b681d>
            let mut safe = Safe::new(Configuration {
                proxy: config::Proxy {
                    factory: CANONICAL.safe_proxy_factory,
                    init_code: CANONICAL.safe_proxy_init_code.to_vec(),
                    singleton: CANONICAL.safe,
                },
                account: config::Account {
                    owners: vec![
                        address!(nz "BF51A8D5ec360F69f9d852Bad1df81585a0b4de2"),
                        address!(nz "84B2D6d9C43Ee780Dd3AA5a7f68aE2A5f45F8206"),
                    ],
                    threshold: 2,
                    setup: Some(config::SafeToL2Setup {
                        address: CANONICAL.safe_to_l2_setup.non_zero().unwrap(),
                        l2_singleton: CANONICAL.safe_l2,
                    }),
                    fallback_handler: CANONICAL.compatibility_fallback_handler.non_zero(),
                    identifier: Some(address!("5afe7A11E7000000000000000000000000000000")),
                },
            });
            safe.update_salt_nonce(|n| *n = [0; 32]);

            let address = safe.creation_address();
            assert_eq!(
                address,
                address!("e47C47CDa5c532A55b930467691BbDB24Ce08bDA")
            );
        }
    }
}

/// The Safe v1.3.0 contract deployments.
///
/// Addresses can be found in the Safe deployments repository:
/// <https://github.com/safe-global/safe-deployments/tree/main/src/assets/v1.3.0>
/// The `proxyCreationCode` can be read from the from the proxy factory:
/// <https://etherscan.io/address/0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2#readContract#F1>
pub mod v1_3_0 {
    use super::*;

    /// The canonical contract deployment.
    pub static CANONICAL: Deployment = Deployment {
        safe_proxy_factory: address!(nz "a6B71E26C5e0845f74c812102Ca7114b6a896AB2"),
        safe_proxy_init_code: &hex!(
            "608060405234801561001057600080fd5b506040516101e63803806101e68339
             818101604052602081101561003357600080fd5b810190808051906020019092
             9190505050600073ffffffffffffffffffffffffffffffffffffffff168173ff
             ffffffffffffffffffffffffffffffffffffff1614156100ca576040517f08c3
             79a0000000000000000000000000000000000000000000000000000000008152
             6004018080602001828103825260228152602001806101c46022913960400191
             505060405180910390fd5b806000806101000a81548173ffffffffffffffffff
             ffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffff
             ffffffffff1602179055505060ab806101196000396000f3fe608060405273ff
             ffffffffffffffffffffffffffffffffffffff600054167fa619486e00000000
             0000000000000000000000000000000000000000000000006000351415605057
             8060005260206000f35b3660008037600080366000845af43d6000803e600081
             14156070573d6000fd5b3d6000f3fea2646970667358221220d1429297349653
             a4918076d650332de1a1068c5f3e07c5c82360c277770b955264736f6c634300
             07060033496e76616c69642073696e676c65746f6e2061646472657373207072
             6f7669646564"
        ),
        safe: address!(nz "d9Db270c1B5E3Bd161E8c8503c55cEABeE709552"),
        safe_l2: address!(nz "3E5c63644E683549055b9Be8653de26E0B4CD36E"),
        safe_to_l2_setup: Address::zero(),
        compatibility_fallback_handler: address!("f48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4"),
    };

    #[cfg(test)]
    mod tests {
        use super::*;
        use deadbeef_core::{config, Configuration, Safe};

        #[test]
        fn proxy_init_code_digest() {
            let proxy = config::Proxy {
                factory: CANONICAL.safe_proxy_factory,
                init_code: CANONICAL.safe_proxy_init_code.to_vec(),
                singleton: CANONICAL.safe,
            };
            assert_eq!(
                proxy.init_code_hash(),
                hex!("56e3081a3d1bb38ed4eed1a39f7729c3cc77c7825794c15bbf326f3047fd779c"),
            );
        }

        #[test]
        fn compute_address() {
            // <https://etherscan.io/tx/0x7b0615b648cb5b9ee366cd22af4e0e40fe90d67c0e140c6efdaabb20b3033a63>
            let mut safe = Safe::new(Configuration {
                proxy: config::Proxy {
                    factory: CANONICAL.safe_proxy_factory,
                    init_code: CANONICAL.safe_proxy_init_code.to_vec(),
                    singleton: CANONICAL.safe,
                },
                account: config::Account {
                    owners: vec![
                        address!(nz "5c8c76f2e990f194462dc5f8a8c76ba16966ed42"),
                        address!(nz "703f28830eeaaad54e786a839f6602ca098016a5"),
                        address!(nz "0e706a98f414f49a412107641c0820b0153ff5dc"),
                        address!(nz "173286fafabea063eeb3726ee5efd4ff414057b9"),
                        address!(nz "2f2806e8b288428f23707a69faa60f52bc565c17"),
                        address!(nz "4507cfb4b077d5dbddd520c701e30173d5b59fad"),
                    ],
                    threshold: 3,
                    setup: None,
                    fallback_handler: CANONICAL.compatibility_fallback_handler.non_zero(),
                    identifier: None,
                },
            });
            safe.update_salt_nonce(|n| {
                n.copy_from_slice(&hex!(
                    "0000000000000000000000000000000000000000000000000000018bbf9209f3"
                ))
            });

            let address = safe.creation_address();
            assert_eq!(
                address,
                address!("5836152812568244760ba356b5f3838aa5b672e0")
            );
        }
    }
}
