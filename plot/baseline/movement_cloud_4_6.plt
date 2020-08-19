clear
reset

set datafile separator "|"

input = '< sqlite3 dat/dat.db "select pos4, pos5, pos6 from dat where batch_id=2 and irun=42;"'

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/movement_cloud_4_6.png"

set xlabel "hp4"
set ylabel "hp5"
set zlabel "hp6"

splot [-100:100] [-100:100] [-1000:1000] input using 1:2:3 t ''

