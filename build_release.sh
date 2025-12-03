# at the time of doing this my CPU is znver2
export RUSTFLAGS="-Zexport-executable-symbols -Ctarget-cpu=native"
cargo build --release -Z build-std="core,panic_abort"
