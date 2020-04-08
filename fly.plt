#vim_filetype gnuplot
set terminal gif animate delay 100
set output 'fly_train.gif'
set xrange [-110:110]
set yrange [-110:110]
do for [i=0:100:1] {
  set title '#'.i
  # plot 'train_dat/p_'.i.'.dat' lc 'black' title 'particle', 'train_dat/l_'.i.'.dat' lc 'red' title 'pareto'
  plot 'train_dat/p_'.i.'.dat' using 1 lc 'black' title 'particle', 'train_dat/l_'.i.'.dat' using 1 lc 'red' title 'pareto'
}
