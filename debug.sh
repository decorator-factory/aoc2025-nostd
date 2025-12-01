set -e

cargo rustc -Zbuild-std -- -C panic=abort
valgrind ./target/debug/aoc2025-nostd $@
