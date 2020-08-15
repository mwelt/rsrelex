clear
reset

inputfile =  "dat/pso_f1_wp_10k_100_50_5_fix_batch/44.dat"

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/movement_cloud_1_3.png"

set xlabel "hp1"
set ylabel "hp2"
set zlabel "hp3"

splot [-100:100] [-100:100] [-1000:1000] inputfile every 1:1:1::50 using 1:2:3 t ''

