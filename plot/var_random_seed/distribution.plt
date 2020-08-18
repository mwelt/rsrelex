clear
reset

set datafile separator "|"

input_baseline = '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=2 and icycle=100 group by irun;"'

input_random = '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=1 and icycle=100 group by irun;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/var_random_seed/distribution.png"

set style fill transparent solid 0.5 noborder

binwidth=0.05
boxwidth=binwidth
bin(x,width)=width*floor(x/width)

set xlabel "F_1"
set ylabel "h_{50/100}(F_1)"

plot input_baseline using (bin($2,binwidth) + binwidth/2):(1.0/50) smooth freq with boxes t 'baseline', input_random using (bin($2,binwidth) + binwidth/2):(1.0/100) smooth freq with boxes t 'random'
