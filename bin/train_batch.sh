#! /usr/bin/env sh

INPUT_FILE=countries.txt
OUTPUT_DIR=pso_f1_wp_10k_100_50_5_batch

NITER=100
NPARTICLES=50
NBWORDS=5

BIN_FILES=~/data/wikipedia/bin_10k/

run_train() {
  $(env RUST_LOG=info \
    ./target/release/rsrelex \
    --train $INPUT_FILE \
    --to $OUTPUT_DIR/$1.dat \
    -b ~/data/wikipedia/bin_10k/ \
    --tniter $NITER \
    --tnparticles $NPARTICLES \
    --tnbwords $NBWORDS 
  )
}

for i in $(seq 11 100) 
do
  run_train $i 
done

