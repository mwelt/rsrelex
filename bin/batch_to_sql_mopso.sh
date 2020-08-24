#! /usr/bin/env sh

IN_DIR=$1
BATCH_ID=$2
DB=$3

process_file(){
  FILE=$1
  RUN=$2

  echo \
"INSERT INTO dat 
  (batch_id, irun, icycle, iparticle, pos1, pos2, pos3, pos4, pos5, pos6, fitness, \"precision\", recall)
VALUES" > t.sql

  cat $FILE \
    | sed '/^[[:space:]]*$/d' \
    | sed 's/NaN/0/g' \
    | awk -v batch_id=$BATCH_ID -v run=$RUN \
    '{
      print "("batch_id", " run", " $1", -1," $2", " $3", " $4", " $5", " $6", " $7", -1, " $8", " $9")"
    }'\
    | sed '$!s/$/,/;$a\;' >> t.sql

    sqlite3 $DB < t.sql
}

BATCH_SIZE=$(ls -A $IN_DIR | wc -l)

echo "BATCH_SIZE: $BATCH_SIZE"

for i in $(seq 1 $BATCH_SIZE)
do
  echo "processing file $IN_DIR/$i.dat"
  process_file $IN_DIR/$i.dat $i 
  rm t.sql
done
