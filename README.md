# `0xdeadbeef`

Tool used for computing vanity Gnosis Safe addresses.

This tool is quite limited in that it only supports the `v1.3.0` Safe deployment:
- `GnosisSafeProxyFactory` [`0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2`](https://etherscan.io/address/0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2)
- `GnosisSafe` [`0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552`](https://etherscan.io/address/0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552)

Since this version of the Safe proxy factory uses `CREATE2` op-code, we can change the final address by fiddling with the 
It works by randomly trying out different values for the `saltNonce` parameter until it find ones that creates

## Building

For longer prefixes, this can take a **very** long time, so be sure to build with release:
```
cargo build --release
```

## Usage

```
deadbeef OWNER_ADDRESS PREFIX
```

For example, to generate calldata for creating a Safe with an initial owner of `0x0102030405060708091011121314151617181920` and prefix `0xdeadbeef`:

```
deadbeef 0x0102030405060708091011121314151617181920 0xdeadbeef
```

This will output some result like:
```
address:    0xdeadbeef57a22ef9446d6c0643308a8cf8ed9fa1
factory:    0xa6b71e26c5e0845f74c812102ca7114b6a896ab2
salt_nonce: 0xd7ed00a81a6f2224c77d7272f7a10fba0d0db7f619f01c90a2365f0f5c1ea42c
calldata:   0x1688f0b9...
```

## Creating the Safe

The above command will generate some calldata for creating a safe with a single inital owner with a 1/1 threshold.
To create the safe, simply execute a transaction to the factory address with the generated calldata.
This can be done in MetaMask directly for example.

Don't forget to configure owners and threshold after creating the Safe!
