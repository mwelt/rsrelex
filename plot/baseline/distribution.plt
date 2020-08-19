clear
reset

set datafile separator "|"

input = '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=2 and icycle=100 group by irun;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/distribution.png"

set style fill transparent solid 0.5 noborder
stats input

binwidth=0.05
boxwidth=binwidth
bin(x,width)=width*floor(x/width)

set xlabel "F_1"
set ylabel "h_{50}(F_1)"

plot input using (bin($2,binwidth) + binwidth/2):(1.0/STATS_records) smooth freq with boxes t ''
