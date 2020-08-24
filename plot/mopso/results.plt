clear
reset

set datafile separator "|"

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/mopso/results.png"

set xlabel "precision"
set ylabel "recall"
# set title "Results"

# separate these two plots in latex
plot '< sqlite3 dat/dat.db "select \"precision\", recall from dat where batch_id=4 and icycle=100;"' \
using 1:2 t '' with points pointtype 13 lc black 

