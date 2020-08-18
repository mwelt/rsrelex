clear
reset

set datafile separator "|"

input_baseline='< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=2 group by icycle;"'

input_8='< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=3 group by icycle;"'

input_11='< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=0 group by icycle;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/var_count_seed/convergence.png"

set xlabel "no. cycle"
set ylabel "F_1"

plot input_baseline using 1:2 t 'baseline' with lines, input_8 using 1:2 t '8' with lines, input_11 using 1:2 t '11' with lines
