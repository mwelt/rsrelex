clear
reset

set datafile separator "|"

# inputfile =  "dat/pso_f1_wp_10k_100_50_5_fix_batch/44.dat"

set terminal pngcairo size 1024, 2000 enhanced font 'Verdana,10'

input1 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=1;"' 
input10 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=10;"' 
input20 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=20;"' 
input30 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=30;"' 
input40 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=40;"' 
input50 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=50;"' 
input60 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=60;"' 
input70 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=70;"' 
input80 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=80;"' 
input90 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=90;"' 
input100 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and icycle=100;"' 

set output "plot/out/baseline/movement_cloud_1_3_1.png"

set xlabel "hp1"
set ylabel "hp2"
set zlabel "hp3"

set multiplot layout 3, 2

splot [-100:100] [-100:100] [-1000:1000] input1 using 1:2:3 t '#1'
splot [-100:100] [-100:100] [-1000:1000] input10 using 1:2:3 t '#10'
splot [-100:100] [-100:100] [-1000:1000] input20 using 1:2:3 t '#20'
splot [-100:100] [-100:100] [-1000:1000] input30 using 1:2:3 t '#30'
splot [-100:100] [-100:100] [-1000:1000] input40 using 1:2:3 t '#40'
splot [-100:100] [-100:100] [-1000:1000] input50 using 1:2:3 t '#50'

unset multiplot

set output "plot/out/baseline/movement_cloud_1_3_2.png"

set xlabel "hp1"
set ylabel "hp2"
set zlabel "hp3"

set multiplot layout 3, 2

splot [-100:100] [-100:100] [-1000:1000] input60 using 1:2:3 t '#60'
splot [-100:100] [-100:100] [-1000:1000] input60 using 1:2:3 t '#70'
splot [-100:100] [-100:100] [-1000:1000] input80 using 1:2:3 t '#80'
splot [-100:100] [-100:100] [-1000:1000] input90 using 1:2:3 t '#90'
splot [-100:100] [-100:100] [-1000:1000] input100 using 1:2:3 t '#100'

unset multiplot
