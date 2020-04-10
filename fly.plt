#vim_filetype gnuplot
set terminal gif animate delay 100
set output 'train_swarm.gif'
set xrange [-110:110]
set yrange [-110:110]
set zrange [-1000:1000]
do for [i=0:100:1] {
  set title 'swarm 0:1 #'.i
  # plot 'train_dat/p_'.i.'.dat' lc 'black' title 'particle', 'train_dat/l_'.i.'.dat' lc 'red' title 'pareto'
  splot 'train_dat/s_'.i.'.dat' using 1:2:3 lc 'black'
}

# set output 'train_fitness.gif'
# set xrange [0:500]
# set yrange [-4:4]
# do for [i=0:100:1] {
#   set title 'fitness #'.i
#   # plot 'train_dat/p_'.i.'.dat' lc 'black' title 'particle', 'train_dat/l_'.i.'.dat' lc 'red' title 'pareto'
#   plot 'train_dat/f_'.i.'.dat' lc 'black' title 'fitness of particle i'
# }
