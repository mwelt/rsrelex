clear
reset

set datafile separator "|"

input1 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and iparticle=1;"' 
input2 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and iparticle=23;"' 
input3 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and iparticle=42;"' 
input4 = '< sqlite3 dat/dat.db "select pos1, pos2, pos3 from dat where batch_id=2 and irun=42 and iparticle=49;"' 

set terminal pngcairo size 1024, 768 enhanced font 'Verdana,10'

set output "plot/out/baseline/movement_particles.png"

set view 67, 164, 0.8, 0.8 

set xlabel "hp1"
set ylabel "hp2"
set zlabel "hp3"

splot [-100:100] [-100:100] [-500:1000]\
input1 using 1:2:3 t 'particle 1' with points pointtype 1,\
input2 using 1:2:3 t 'particle 23' with points pointtype 1,\
input3 using 1:2:3 t 'particle 42' with points pointtype 1,\
input4 using 1:2:3 t 'particle 49' with points pointtype 1
