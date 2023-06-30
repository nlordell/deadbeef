# WebAssembly `0xdeadbeef`

The `deadbeef` tool packaged for the Web using WebAssembly.

## Requirements

- `wasm-pack`
- `wasm-opt`
- `deno`

## Building

As the build process is non-trivial, a `Makefile` is provided.
To build the WebAssembly package, just run:

```sh
make
```

## Testing

Easy-peasy-lemon-squeezy:

```sh
make test
```

## Example

An example web page that computes a Safe creation transaction for a single owner is provided.
To check it out, navigate to [`http://localhost:8000/example`](http://localhost:8000/example) while running:

```sh
make host
```
