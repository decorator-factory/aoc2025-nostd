cargo rustc --quiet --release -Zbuild-std -- -C panic=abort
./target/release/aoc2025-nostd $@
