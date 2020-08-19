#! /usr/bin/env sh

INPUT_FILE=countries.txt
OUTPUT_DIR=pso_f1_wp_10k_100_50_11_batch

NITER=100
NPARTICLES=50
NBWORDS=5

BIN_FILES=~/data/wikipedia/bin_10k/

run_train() {

  # $(env RUST_LOG=info \
  #   ./target/release/rsrelex \
  #   --train $INPUT_FILE \
  #   --to $OUTPUT_DIR/$1.dat \
  #   -b ~/data/wikipedia/bin_10k/ \
  #   --tniter $NITER \
  #   --tnparticles $NPARTICLES \
  #   --tnbwords $NBWORDS 
  # )

  $(cat countries_seed.txt | tail -n8 | env RUST_LOG=info \
    ./target/release/rsrelex \
    --train $INPUT_FILE \
    --to $OUTPUT_DIR/$1.dat \
    -b ~/data/wikipedia/bin_10k/ \
    --tniter $NITER \
    --tnparticles $NPARTICLES \
    --tnbwords $NBWORDS 
  )
}

for i in $(seq 1 50) 
do
  run_train $i 
done

