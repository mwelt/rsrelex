clear
reset

set datafile separator "|"

input_500  = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=10 group by icycle;"'
input_1k   = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=11 group by icycle;"'
input_5k   = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=12 group by icycle;"'
input_10k  = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=2 group by icycle;"'
input_50k  = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=14 group by icycle;"'
input_100k = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=15 group by icycle;"'
input_500k = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=16 group by icycle;"'
input_1m   = '< sqlite3 dat/dat.db "select icycle, avg(fitness) from dat where batch_id=17 group by icycle;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/var_corpus_size/convergence.png"

set xlabel "no. cycle"
set ylabel "F_1"
# set title "Results"

plot input_500 using 1:2 t '500' with lines, \
input_1k using 1:2 t '1k' with lines, \
input_5k using 1:2 t '5k' with lines, \
input_10k using 1:2 t 'basline (10k)' with lines, \
input_50k using 1:2 t '50k' with lines, \
input_100k using 1:2 t '100k' with lines, \
input_500k using 1:2 t '500k' with lines, \
input_1m using 1:2 t '1m' with lines
