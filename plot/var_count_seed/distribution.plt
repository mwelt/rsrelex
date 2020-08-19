clear
reset

set datafile separator "|"

input_baseline = '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=2 and icycle=100 group by irun;"'

input_8 = '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=3 and icycle=100 group by irun;"'

input_11 = '< sqlite3 dat/dat.db "select irun, max(fitness) from dat where batch_id=0 and icycle=100 group by irun;"'

# set style histogram cluster gap 1
set style fill transparent solid 0.5 noborder

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/var_count_seed/distribution.png"

# stats input_baseline

binwidth=0.05
boxwidth=binwidth
bin(x,width)=width*floor(x/width)

set xlabel "F_1"
set ylabel "h_{50}(F_1)"

plot input_baseline using (bin($2,binwidth) + binwidth/2):(1.0/50) smooth freq with boxes t 'baseline (5)', input_8 using (bin($2,binwidth) + binwidth/2):(1.0/50) smooth freq with boxes t '8', input_11 using (bin($2,binwidth) + binwidth/2):(1.0/50) smooth freq with boxes t '11'
