# aoc2025-nostd

Advent of Code 2025, using Rust in `no_std` mode.
Let's see how far I can get before giving up.

## How to run this

I ran into some bugs/anomalies with build-std, so you have to run `./build_release.sh` and `./build_debug.sh` to build the program, and then `target/<debug/release>/aoc2025-nostd day1a inputs/day1_example.txt` to run it.

Running unit tests fortunately works with `cargo test`.
