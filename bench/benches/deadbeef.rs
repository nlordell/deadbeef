use deadbeef_core::{address, config, hex, Configuration, Safe};

fn main() {
    divan::main();
}

fn safe() -> Safe {
    Safe::new(Configuration {
        proxy: config::Proxy {
            factory: address!(nz "fafafafafafafafafafafafafafafafafafafafa"),
            init_code: hex!(
                "c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de
                 c0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0dec0de"
            )
            .to_vec(),
            singleton: address!(nz "5afe5afe5afe5afe5afe5afe5afe5afe5afe5afe"),
        },
        account: config::Account {
            owners: vec![address!(nz "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")],
            threshold: 1,
            setup: Some(config::SafeToL2Setup {
                address: address!(nz "5e795e795e795e795e795e795e795e795e795e79"),
                l2_singleton: address!(nz "5afe125afe125afe125afe125afe125afe125afe"),
            }),
            fallback_handler: Some(address!(nz "fa11baccfa11baccfa11baccfa11baccfa11bacc")),
            identifier: None,
        },
    })
}

#[divan::bench]
fn check(bencher: divan::Bencher) {
    let mut safe = safe();
    let prefix = hex!("deadbeef");

    bencher.bench_local(move || {
        deadbeef_core::search_iter(&mut safe, &divan::black_box(prefix), |n| {
            n.copy_from_slice(&divan::black_box([0xee; 32]));
        });
    });
}
