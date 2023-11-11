# `0xdeadbeef`

Tool used for computing vanity Safe addresses.

This tool only officially supports the latest Safe deployment [`v1.3.0`](https://github.com/safe-global/safe-deployments/tree/main/src/assets/v1.3.0).
For Ethereum:
- `SafeProxyFactory` [`0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2`](https://etherscan.io/address/0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2)
- `Safe` [`0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552`](https://etherscan.io/address/0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552)
- `CompatibilityFallbackHandler` [`0xf48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4`](https://etherscan.io/address/0xf48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4)

Since this version of the Safe proxy factory uses `CREATE2` op-code, we can change the final address by fiddling with the user-specified `saltNonce` parameter.
It works by randomly trying out different values for the `saltNonce` parameter until it find ones that creates an address matching the desired prefix.

## Building

For longer prefixes, this can take a **very** long time, so be sure to build with release:

```
cargo build --release
```

## Usage

```
deadbeef --help
```

For example, to generate calldata for creating a Safe with initial owners of `0x1111111111111111111111111111111111111111` and `0x2222222222222222222222222222222222222222` and prefix `0xdeadbeef`:

```
deadbeef \
  --owner 0x1111111111111111111111111111111111111111 \
  --owner 0x2222222222222222222222222222222222222222 \
  --prefix 0xdeadbeef
```

Note that the owner signature threshold defaults to 1 but can optionally be specified with:

```
deadbeef ... --threshold 2 ...
```

This will output some result like:

```
address:   0xdeaDbeef17411A25478b0a27BE1aD71533FB9E79
factory:   0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2
singleton: 0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552
fallback:  0xf48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4
owners:    0x1111111111111111111111111111111111111111
           0x2222222222222222222222222222222222222222
threshold: 1
calldata:  0x1688f0b9000000000000000000000000d9db270c1b5e3bd161e8c8503c55ceabee7095520000000000000000000000000000000000000000000000000000000000000060496bc3e394a70708faee280d08458bea16b2a0d43b3b2d02cebcdf5714619c710000000000000000000000000000000000000000000000000000000000000184b63e800d0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000160000000000000000000000000f48f2b2d2a534e402487b3ee7c18c33aec0fe5e4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000011111111111111111111111111111111111111110000000000000000000000002222222222222222222222222222222222222222000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

For using Safe deployments on different chains can also be used:

```
deadbeef ... --chain 100 ...
```

As well as custom fallback handlers:

```
deadbeef ... --fallback-handler 0x4e305935b14627eA57CBDbCfF57e81fd9F240403 ...
```

## Creating the Safe

The above command will generate some [calldata](https://www.quicknode.com/guides/ethereum-development/transactions/ethereum-transaction-calldata) for creating a Safe with the specified owners and threshold.

To create the safe, simply execute a transaction to the [factory address](https://etherscan.io/address/0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2) with the generated calldata, or use the `createProxyWithNonce` function on Etherscan.
The transaction can be executed from any account (it can be done in MetaMask directly for example).

### Metamask Steps

Go to Settings -> Advanced and enable `Show hex data`. When you go to create a transaction you will have a new optional field labelled `Hex data`.

Send a 0Ξ transaction to the factory address, placing the generated calldata in the `Hex data` field.

Metamask will recognise it as a contract interaction in the confirmation step.

### Etherscan

Use the `--params` flag to output contract-ready inputs.

1. Visit the `factory` URL from the command output. The link should open up the explorer at the correct location, if not click on _Contract_ > _Write Contract_.
2. Click on _Connect to Web3_ to connect the account you wish to pay for the Safe creation.
3. Fill the fields for the function _3. createProxyWithNonce (0x1688f0b9)_ using the generated outputs.

## Unsupported Chains

Safe deployments on non-officially supported networks can also be used by overriding all contract addresses and the proxy init code:

```
deadbeef ... \
  --chain $UNSUPPORTED_CHAIN \
  --proxy-factory 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --proxy-init-code 0xbb \
  --singleton 0xcccccccccccccccccccccccccccccccccccccccc \
  --fallback-handler 0xdddddddddddddddddddddddddddddddddddddddd
```

**Use this with caution**, this assumes that the proxy address is computed in the exact same was as on Ethereum, which may not be the case for all networks.
This feature is not officially supported by the tool.

## Is This Vegan Friendly 🥦?

Of course!
No actual cows were harmed in the creation or continual use of this tool.

```
% alias deadbeef=seedfeed
% seedfeed \
  --owner 0x1111111111111111111111111111111111111111 \
  --owner 0x2222222222222222222222222222222222222222 \
  --owner 0x3333333333333333333333333333333333333333 \
  --threshold 2 \
  --prefix 0x5eedfeed
address:   0x5EedFeED446B211419EBac9253FbB8b9556781D1
factory:   0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2
singleton: 0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552
fallback:  0xf48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4
owners:    0x1111111111111111111111111111111111111111
           0x2222222222222222222222222222222222222222
           0x3333333333333333333333333333333333333333
threshold: 2
calldata:  0x1688f0b9000000000000000000000000d9db270c1b5e3bd161e8c8503c55ceabee70955200000000000000000000000000000000000000000000000000000000000000605b237da2310f4948260ac5661d88f5150e5e62ca1f4faa76e3598bd427f212e800000000000000000000000000000000000000000000000000000000000001a4b63e800d0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000180000000000000000000000000f48f2b2d2a534e402487b3ee7c18c33aec0fe5e40000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000111111111111111111111111111111111111111100000000000000000000000022222222222222222222222222222222222222220000000000000000000000003333333333333333333333333333333333333333000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```
