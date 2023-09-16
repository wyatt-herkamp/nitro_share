# Nitro Share Frontend
The frontend for the Nitro Share web application


## Requirements
- Node 20
- Vite
- NPM
- Cargo for building the types from the backend (see below)

### Building Types
Types are generated using [typeshare](https://github.com/1Password/typeshare) and the following command:

Specifically the [fork](https://github.com/tomjw64/typeshare) by [tomjw64](https://github.com/tomjw64)

To install this fork run

```console
cargo install --git https://github.com/tomjw64/typeshare --branch allow-override-for-disallowed-types
```

To generate the types, run
```console
typeshare  --lang=typescript --output-file=nitro-share-frontend/src/types.ts ./server ./crates/entities/ ./crates/common
```