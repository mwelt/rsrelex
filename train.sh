env RUST_LOG=info rsrelex --train countries.txt --to pso_f1_wp_1k_100_100_5.dat -b ~/data/wikipedia/bin_1k/
env RUST_LOG=info ./target/release/rsrelex --train countries.txt --to results/pso_f1_wp_10k_100_50_5_2.dat -b ~/data/wikipedia/bin_10k/ --tniter 100 --tnparticles 50 --tnbwords 5
