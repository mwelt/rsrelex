#! /usr/bin/env bash

./target/release/rsrelex --bin-files $2 --import-xml $1 --xt "text" --xl $3 --xp "wikitext::strip_markup"
