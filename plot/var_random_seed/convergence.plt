clear
reset

set datafile separator "|"

input_baseline='< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=2 group by icycle;"'

input_random='< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=1 group by icycle;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/var_random_seed/convergence.png"

set xlabel "no. cycle"
set ylabel "F_1"
# set title "Results"

plot input_baseline using 1:2 t 'baseline' with lines, input_random using 1:2 t'random' with lines
