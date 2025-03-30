use deadbeef_core::{config, Configuration, Safe};
use hex::FromHexError;
use std::error::Error;
use wasm_bindgen::prelude::*;
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

mod js {
    use serde::{de, Deserialize, Serialize};

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Configuration {
        pub proxy_factory: String,
        pub proxy_init_code: String,
        pub singleton: String,
        pub owners: Vec<String>,
        pub threshold: usize,
        #[serde(deserialize_with = "deserialize_setup", flatten)]
        pub setup: Option<Setup>,
        #[serde(default)]
        pub fallback_handler: Option<String>,
    }

    pub struct Setup {
        pub safe_to_l2_setup: String,
        pub l2_singleton: String,
    }

    fn deserialize_setup<'de, D>(deserializer: D) -> Result<Option<Setup>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Inner {
            safe_to_l2_setup: Option<String>,
            l2_singleton: Option<String>,
        }

        let setup = Inner::deserialize(deserializer)?;
        match (setup.safe_to_l2_setup, setup.l2_singleton) {
            (Some(safe_to_l2_setup), Some(l2_singleton)) => Ok(Some(Setup {
                safe_to_l2_setup,
                l2_singleton,
            })),
            (None, None) => Ok(None),
            (None, _) => Err(serde::de::Error::missing_field("safeToL2Setup")),
            (_, None) => Err(serde::de::Error::missing_field("l2Singleton")),
        }
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Creation {
        pub creation_address: String,
        pub salt_nonce: String,
        pub transaction: Transaction,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Transaction {
        pub to: String,
        pub calldata: String,
    }
}

#[wasm_bindgen]
pub fn search(safe: JsValue, prefix: &str) -> Result<JsValue, String> {
    let result = inner(safe, prefix);

    // TODO(nlordell): Ideally, we would just return `Result<JsValue, JsError>`
    // and be done with it. However, it looks like `wasm-pack`/`wasm-bindgen` is
    // not marshalling the `JsError` type correctly, so work around it by using
    // a `String` error and emitting an actual `Error` on the JS side.
    result.map_err(|err| err.to_string())
}

fn inner(safe: JsValue, prefix: &str) -> Result<JsValue, Box<dyn Error>> {
    let config = serde_wasm_bindgen::from_value::<js::Configuration>(safe)?;
    let prefix = hex_decode(prefix)?;

    let mut safe = Safe::new(Configuration {
        proxy: config::Proxy {
            factory: config.proxy_factory.parse()?,
            init_code: hex_decode(&config.proxy_init_code)?,
            singleton: config.singleton.parse()?,
        },
        account: config::Account {
            owners: config
                .owners
                .iter()
                .map(|owner| owner.parse())
                .collect::<Result<Vec<_>, _>>()?,
            threshold: config.threshold,
            setup: config
                .setup
                .as_ref()
                .map(|setup| {
                    setup.safe_to_l2_setup.parse().and_then(|safe_to_l2_setup| {
                        Ok(config::SafeToL2Setup {
                            address: safe_to_l2_setup,
                            l2_singleton: setup.l2_singleton.parse()?,
                        })
                    })
                })
                .transpose()?,
            fallback_handler: config
                .fallback_handler
                .map(|fallback_handler| fallback_handler.parse())
                .transpose()?,
            identifier: None,
        },
    });

    deadbeef_core::search(&mut safe, &prefix);

    let transaction = safe.transaction();
    let creation = serde_wasm_bindgen::to_value(&js::Creation {
        creation_address: safe.creation_address().to_string(),
        salt_nonce: hex_encode(&safe.salt_nonce()),
        transaction: js::Transaction {
            to: transaction.to.to_string(),
            calldata: hex_encode(&transaction.calldata),
        },
    })?;

    Ok(creation)
}

fn hex_decode(s: &str) -> Result<Vec<u8>, FromHexError> {
    hex::decode(s.strip_prefix("0x").unwrap_or(s))
}

fn hex_encode(b: &[u8]) -> String {
    format!("0x{}", hex::encode(b))
}
