clear
reset

# inputfile = "dat/pso_f1_wp_10k_100_50_5_fix_batch_winner.dat"
set datafile separator "|"

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/results.png"

set xlabel "no. run"
set ylabel "F_1"
# set title "Results"

# separate these two plots in latex
plot '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=2 and icycle=100 group by irun;"' \
using 1:2 t '' with points pointtype 13 lc black 

