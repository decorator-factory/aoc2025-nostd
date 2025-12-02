export RUSTFLAGS="-Zexport-executable-symbols"
cargo build -Zbuild-std="core,panic_abort"
