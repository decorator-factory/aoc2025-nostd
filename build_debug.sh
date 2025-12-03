export RUSTFLAGS="-Zexport-executable-symbols -Cforce-frame-pointers=yes"
cargo build -Zbuild-std="core,panic_abort"
