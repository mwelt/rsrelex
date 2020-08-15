clear
reset

# inputfile = "dat/pso_f1_wp_10k_100_50_5_fix_batch_winner.dat"
set datafile separator "|"

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/distribution_sql.png"

stats inputfile 

binwidth=0.1
boxwidth=binwidth
bin(x,width)=width*floor(x/width)

# set xrange [0:1] noextend

# set title "Distribution"
set xlabel "F_1"
set ylabel "h_{50}(F_1)"

plot inputfile using (bin($7,binwidth) + binwidth/2):(1.0/STATS_records) smooth freq with boxes t ''

