set terminal gif animate delay 100
set output 'fly.gif'
set xrange [-110:110]
set yrange [-110:110]
do for [i=0:100:1] {
  set title '#'.i
  plot 'fly_test/sp_'.i.'.dat' lc 'black' title 'particle', 'fly_test/spl_'.i.'.dat' lc 'red' title 'pareto' 
}
