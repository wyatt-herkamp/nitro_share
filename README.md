# Nitro Share
A file sharing service that is supposed to be a mix of PasteBin and Imgur. 


## Requirements
- Postgres

## Development

### Rust Format
Please use nightly rustfmt to format the code
```console
cargo +nightly fmt
```

## Project Struct
```console
.
??? crates - Crates that contain code for the web server /
?   ??? common Helper Libraries and Types that are re used within the project
?   ??? entities The Database Entities types for sea_orm
?   ??? migrations Migration stuff for sea_orm
?   ??? helper_macros macros used by the server crate to help with the project
??? server - The web server for nitro_share. 
??? frontend - The web server Frontend
??? cli One crate outputs two binaries/
    ??? nitro_share_admin Runs on the server to do any mantiance on the nitro_share server
    ??? nitro_share(ns) A CLI used to upload and retreive stuff from the server
```