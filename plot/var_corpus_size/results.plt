clear
reset

set datafile separator "|"

input = '< sqlite3 dat/dat.db "select b.batch_id, b.ncorpus, d.irun, max(d.fitness) from dat d left join batch b on d.batch_id = b.batch_id where d.batch_id in (10, 11, 12, 2, 14, 15, 16, 17) and d.icycle = 100 group by b.batch_id order by b.ncorpus;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/var_corpus_size/results.png"

set style fill transparent solid 0.5 noborder

set xlabel "corpus size"
set ylabel "F_1"
# set xtics

plot [] [0:1] input using 4:xticlabels(2) with lines t ''
