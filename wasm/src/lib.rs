use deadbeef_core::{Contracts, Safe};
use hex::FromHexError;
use std::error::Error;
use wasm_bindgen::prelude::*;
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

mod js {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Safe {
        pub proxy_factory: String,
        pub proxy_init_code: String,
        pub singleton: String,
        pub fallback_handler: String,
        pub owners: Vec<String>,
        pub threshold: usize,
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
    let safe = serde_wasm_bindgen::from_value::<js::Safe>(safe)?;

    let contracts = Contracts {
        proxy_factory: safe.proxy_factory.parse()?,
        proxy_init_code: hex_decode(&safe.proxy_init_code)?,
        singleton: safe.singleton.parse()?,
        fallback_handler: safe.fallback_handler.parse()?,
    };

    let owners = safe
        .owners
        .iter()
        .map(|owner| owner.parse())
        .collect::<Result<Vec<_>, _>>()?;
    let threshold = safe.threshold;
    let prefix = hex_decode(prefix)?;

    let mut safe = Safe::new(contracts, owners, threshold);
    deadbeef_core::search(&mut safe, &prefix, false);

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
