clear
reset

inputfile =  "dat/pso_f1_wp_10k_100_50_5_fix_batch/44.dat"

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/movement_particles.png"

set view 67, 164, 0.8, 0.8 

set xlabel "hp1"
set ylabel "hp2"
set zlabel "hp3"

splot [-100:100] [-100:100] [-500:1000]\
inputfile every ::1::1 using 1:2:3 t 'particle 1' with points pointtype 1,\
inputfile every ::23::23 using 1:2:3 t 'particle 23' with points pointtype 1,\
inputfile every ::42::42 using 1:2:3 t 'particle 42' with points pointtype 1,\
inputfile every ::49::49 using 1:2:3 t 'particle 49' with points pointtype 1
