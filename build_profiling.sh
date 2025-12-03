# at the time of doing this my CPU is znver2
export RUSTFLAGS="-Zexport-executable-symbols -Cforce-frame-pointers=yes -Ctarget-cpu=native"
cargo build --profile profiling -Z build-std="core,panic_abort"
