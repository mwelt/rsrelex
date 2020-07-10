#! /usr/bin/env sh

IN_DIR=$1
OUT_FILE=$2

get_winner_last_block(){
  FILE=$1

  res=$(tail -n52 $FILE \
    | head -n 50 \
    | awk '{print $1,$2,$3,$4,$5,$6,$7,$21,$22}' \
    | sort -k7 -n \
    | tail -n1)
}

for file in $(ls -A $IN_DIR)
do
  get_winner_last_block $IN_DIR/$file
  echo $res >> $OUT_FILE
done

