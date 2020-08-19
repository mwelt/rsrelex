clear
reset

set datafile separator "|"

input='< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=2 group by icycle;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/convergence.png"

set xlabel "no. cycle"
set ylabel "F_1"
# set title "Results"

plot input using 1:2 t '' with lines lc black 
