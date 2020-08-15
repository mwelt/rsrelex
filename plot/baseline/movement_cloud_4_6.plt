clear
reset

inputfile =  "dat/pso_f1_wp_10k_100_50_5_fix_batch/44.dat"

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/movement_cloud_4_6.png"

set xlabel "hp4"
set ylabel "hp5"
set zlabel "hp6"

splot [-100:100] [-100:100] [-1000:1000] inputfile every 1:1:1::50 using 4:5:6 t ''

