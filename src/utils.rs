use super::*;
use super::types::{Env};
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use log::debug;

pub fn read_word_file(file_name: &str, env: &Env) -> Vec<WordNr> {
    let f = File::open(file_name)
        .unwrap_or_else(|_| panic!("Unable to open word file \"{}\".", file_name));

    let lines: Vec<String> = BufReader::new(f).lines()
        .collect::<Result<Vec<String>>>()
        .unwrap_or_else(|_| panic!("Unable to read word file \"{}\".", file_name));

    let mut count_missing: usize = 0;

    let word_nrs: Vec<WordNr> = lines.iter()
        .filter_map(|s| {
            let o_wnr = env.dict.get_opt_nr(s);
            if o_wnr.is_none() {
                debug!("Reference word \"{}\" not found in dictionary", s);
                count_missing += 1;
            }
            o_wnr
        }).collect();
   
    let x: usize = word_nrs.len() + count_missing;
    info!("{} from {} known words found in word file \"{}\".", 
        word_nrs.len(), x, file_name);

    word_nrs
}

