#!/bin/sh

cargo run --release --bin generate_data
cargo run --release --bin driver -- sample_input.txt > out_singlecore.txt
cargo run --release --bin driver_threaded -- sample_input.txt > out_multicore.txt

sort -k1.1,1.1 out_singlecore.txt > sorted_singlecore.txt
sort -k1.1,1.1 out_multicore.txt > sorted_multicore.txt
diff sorted_singlecore.txt sorted_multicore.txt
