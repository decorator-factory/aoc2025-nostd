export RUSTFLAGS="-Zexport-executable-symbols -Cforce-frame-pointers=yes"
cargo build --release -Z build-std="core,panic_abort"
