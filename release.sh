set -e

cargo rustc --quiet --release -Zbuild-std -- -C panic=abort
time ./target/release/aoc2025-nostd $@
