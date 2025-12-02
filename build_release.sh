export RUSTFLAGS="-Zexport-executable-symbols"
cargo build --release -Z build-std="core,panic_abort"
