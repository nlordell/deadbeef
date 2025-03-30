# `0xdeadbeef`

Tool used for computing vanity Safe addresses.

This tool only officially supports the latest Safe deployment [`v1.4.1`](https://github.com/safe-global/safe-deployments/tree/main/src/assets/v1.4.1).

Since this version of the Safe proxy factory uses `CREATE2` op-code, we can change the final address by fiddling with the user-specified `saltNonce` parameter.
It works by randomly trying out different values for the `saltNonce` parameter until it find ones that creates an address matching the desired prefix.

## Building

For longer prefixes, this can take a **very** long time, so be sure to build with release:

```sh
cargo build --release
```

## Usage

```sh
deadbeef --help
```

For example, to generate calldata for creating a Safe with initial owners of `0x1111111111111111111111111111111111111111` and `0x2222222222222222222222222222222222222222` and prefix `0xdeadbeef`:

```sh
deadbeef \
  --owner 0x1111111111111111111111111111111111111111 \
  --owner 0x2222222222222222222222222222222222222222 \
  --prefix 0x5afe
```

This will output some result like:

```
address:     0x5AFE941f405085500803E7479cEcEC46cB50B70A
factory:     0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67
singleton:   0x41675C099F32341bf84BFc5382aF534df5C7461a
initializer: 0xb63e800d00000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000bd89a1ce4dde368ffab0ec35506eece0b1ffdc540000000000000000000000000000000000000000000000000000000000000160000000000000000000000000fd0732dc9e303f09fcef3a7388ad10a83459ec990000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000111111111111111111111111111111111111111100000000000000000000000022222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000024fe51f64300000000000000000000000029fcb43b46531bca003ddc8fcb67ffe91900c76200000000000000000000000000000000000000000000000000000000
salt nonce:  0xdc50d6fbe25fc5fe0cffb1a2fc5170b76ac620f74f22c7b247d8c73d83072302
---
owners:      0x1111111111111111111111111111111111111111
             0x2222222222222222222222222222222222222222
threshold:   1
to:          0xBD89A1CE4DDe368FFAB0eC35506eEcE0b1fFdc54
data:        0xfe51f64300000000000000000000000029fcb43b46531bca003ddc8fcb67ffe91900c762
fallback:    0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99
---
calldata:    0x1688f0b900000000000000000000000041675c099f32341bf84bfc5382af534df5c7461a0000000000000000000000000000000000000000000000000000000000000060dc50d6fbe25fc5fe0cffb1a2fc5170b76ac620f74f22c7b247d8c73d8307230200000000000000000000000000000000000000000000000000000000000001c4b63e800d00000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000bd89a1ce4dde368ffab0ec35506eece0b1ffdc540000000000000000000000000000000000000000000000000000000000000160000000000000000000000000fd0732dc9e303f09fcef3a7388ad10a83459ec990000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000111111111111111111111111111111111111111100000000000000000000000022222222222222222222222222222222222222220000000000000000000000000000000000000000000000000000000000000024fe51f64300000000000000000000000029fcb43b46531bca003ddc8fcb67ffe91900c7620000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

Note that the owner signature threshold defaults to 1 but can optionally be specified with:

```sh
deadbeef ... --threshold 2 ...
```

For using Safe deployments on different chains can also be used:

```sh
deadbeef ... --chain 100 ...
```

As well as custom fallback handlers:

```sh
deadbeef ... --fallback-handler 0x4e305935b14627eA57CBDbCfF57e81fd9F240403 ...
```

By default, the generated initializer will use the `SafeToL2Setup` contract. This ensures that the Safe deployment transaction can be replayed to get the same address on all supported chains. In order to disable this behaviour (not recommended), set the `--safe-to-l2-setup` flag to the 0 address:

```sh
deadbeef ... --safe-to-l2-setup 0x0000000000000000000000000000000000000000 ...
```

## Creating the Safe

The above command will generate some [calldata](https://www.quicknode.com/guides/ethereum-development/transactions/ethereum-transaction-calldata) for creating a Safe with the specified owners and threshold.

To create the safe, simply execute a transaction to the [factory address](https://etherscan.io/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67) with the generated calldata, or use the `createProxyWithNonce` function on Etherscan.
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

```sh
deadbeef ... \
  --chain $UNSUPPORTED_CHAIN \
  --proxy-factory 0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --proxy-init-code 0xbb \
  --singleton 0xcccccccccccccccccccccccccccccccccccccccc
```

**Use this with caution**, this assumes that the proxy address is computed in the exact same was as on Ethereum, which may not be the case for all networks.
This feature is not officially supported by the tool.

## Is This Vegan Friendly 🥦?

Of course!
No actual cows were harmed in the creation or continual use of this tool.

```sh
alias deadbeef=seedfeed
```
