## Building

```sh
  $ cargo wasm
```

## Testing

```sh
  $ cargo unit-test
```

## Compiling

```sh
  $ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 .
```
