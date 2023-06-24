# `0xdeadbeef`

Tool used for computing vanity Gnosis Safe addresses.

This tool is currently hard-coded to only support the [`v1.4.1`](https://github.com/safe-global/safe-deployments/tree/9cf5d5f75819371b7b63fcc66f316bcd920f3c58/src/assets/v1.4.1) Safe deployment:
- `SafeProxyFactory` [`0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67`](https://etherscan.io/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67)
- `GnosisSafe` [`0x41675C099F32341bf84BFc5382aF534df5C7461a`](https://etherscan.io/address/0x41675C099F32341bf84BFc5382aF534df5C7461a)
- `CompatibilityFallbackHandler` [`0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99`](https://etherscan.io/address/0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99)

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
address:   0xdEaDBeeFdE563691c0Db36bc27e3fb45358F63A6
factory:   0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67
singleton: 0x41675C099F32341bf84BFc5382aF534df5C7461a
fallback:  0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99
owners:    0x00000000D3226dC8A29cBb43C839814a551f91cd
           0x00000000D4707c8c11Cc0F85A4E264729B94813d
threshold: 2
calldata:  0x1688f0b900000000000000000000000041675c099f32341bf84bfc5382af534df5c7461a00000000000000000000000000000000000000000000000000000000000000600a9528d57fdbbb0fa736a49dc800aec4d92522da2ee0218a5e3ab64d72cfce6a0000000000000000000000000000000000000000000000000000000000000184b63e800d0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000160000000000000000000000000fd0732dc9e303f09fcef3a7388ad10a83459ec99000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000d3226dc8a29cbb43c839814a551f91cd00000000000000000000000000000000d4707c8c11cc0f85a4e264729b94813d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

## Creating the Safe

The above command will generate some calldata for creating a Safe with the specified owners and threshold.
To create the safe, simply execute a transaction to the [factory address](https://etherscan.io/address/0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67) with the generated calldata.
The transaction can be executed from any account (it can be done in MetaMask directly for example).

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
factory:   0x4e1DCf7AD4e460CfD30791CCC4F9c8a4f820ec67
singleton: 0x41675C099F32341bf84BFc5382aF534df5C7461a
fallback:  0xfd0732Dc9E303f09fCEf3a7388Ad10A83459Ec99
owners:    0x1111111111111111111111111111111111111111
           0x2222222222222222222222222222222222222222
           0x3333333333333333333333333333333333333333
threshold: 2
calldata:  0x1688f0b900000000000000000000000041675C099F32341bf84BFc5382aF534df5C7461a00000000000000000000000000000000000000000000000000000000000000605b237da2310f4948260ac5661d88f5150e5e62ca1f4faa76e3598bd427f212e800000000000000000000000000000000000000000000000000000000000001a4b63e800d0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000180000000000000000000000000fd0732Dc9E303f09fCEf3a7388Ad10A83459Ec990000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000111111111111111111111111111111111111111100000000000000000000000022222222222222222222222222222222222222220000000000000000000000003333333333333333333333333333333333333333000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```
