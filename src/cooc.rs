use super::types::{AsyncLogger, CoocInput, EMPTY_WORD,
WordNr, SentenceId, Env, WPair, Pattern}; 

use std::collections::HashMap;
use std::collections::HashSet;

// how many bootstrap words share this cooc
const COOC1_WORD_FREQUENCY_BOOST: i16 = 10;
// how frequent is this cooc overall w.r.t bootstrap set
const COOC1_SET_FREQUENCY_BOOST: i16 = 1;
// overall termfrequency i.e. how many sentences contain this term
const COOC1_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE: f32 = -0.1;

// // pattern was found for one or more wpairs 
// const PATTERN_WPAIR_BOOST: i16 = 10;
// // pattern (infix) was found one or more times
// const PATTERN_PATTERN_BOOST: i16 = -2;
// // pattern is to short MALUS
// const PATTERN_SHORT_SIZED_BOOST: i16 = -40;
// // pattern is medium sized
// const PATTERN_MEDIUM_SIZED_BOOST: i16 = 0;
// // pattern is to long MALUS
// const PATTERN_LONG_SIZED_BOOST: i16 = -40;

// const PATTERN_SURVIVOR_THRESHOLD: i16 = 12;

// const WPAIR_SURVIVOR_THRESHOLD: i16 = 20;

// // word appears frequently in the global corpus
// // needs to be dependent on the size of the corpus
// const WPAIR_WORD_GLOBAL_FREQUENCY_BOOST_PER_SENTENCE: f32 = -0.1; 

// // wpair is identified over various patterns
// const WPAIR_PATTERN_BOOST: i16 = 10;

fn cooccurrences_for_word(word: WordNr, env: &Env) -> HashMap<WordNr, u32>{
    
    // get all sentences which contain word
    let sentence_ids = env.get_inverted_idx(&word);

    let mut word_on_count: HashMap<WordNr, u32> = HashMap::new(); 

    // count co-occurrences
    for s_id in sentence_ids {
        for w_nr in env.get_sentence(s_id) {
            let current_count = word_on_count.entry(*w_nr)
                .or_insert(0);
            *current_count += 1;
        }
    }
    
    word_on_count

}

fn cooc_input_to_word_nr_set(cooc_input: &CoocInput, env: &Env) 
    -> HashSet<WordNr>{

    cooc_input.set.iter()
        .map(|word_str| {
            let opt_word_nr = env.dict.get_opt_nr(word_str);
            if opt_word_nr.is_none() {
                println!("Word \"{}\" not found in dictionary 
                         - removing it from bootstrap set.", word_str);
            }
            opt_word_nr
        })
        .filter(|opt_word_nr| match opt_word_nr {
            None => {
                    false
            }
            Some(_) => { true }
        })
        .map(|opt_word_nr| opt_word_nr.unwrap())
        .collect()
}

// #[derive(Debug)]
// struct CoocStats {
//     // how many bootstrap words share this cooc
//     word_frequency: u8,
//     // how frequent is this cooc overall w.r.t bootstrap set
//     set_frequency: u32,
//     // overall termfrequency i.e. how many sentences contain this term
//     term_frequency: usize
// }

pub fn do_cooc(cooc_input: CoocInput, env: &Env) {

    println!("Converting input {:?} into set of word numbers", cooc_input);
    let bootstrap_set = cooc_input_to_word_nr_set(&cooc_input, &env);
    println!("Done converting input into set of word numbers: {:?}",           
             bootstrap_set);

    let mut coocs_on_cooc_stat: HashMap<WordNr, CoocStats> = HashMap::new(); 

    println!("Collecting syntagmatic context");
    for word in bootstrap_set {
        let coocs_for_word = cooccurrences_for_word(word, env);
        let mut already_word_frequency_inced: HashSet<WordNr> = HashSet::new(); 

        for (cooc, count) in coocs_for_word {
            let mut current_cooc_stat = coocs_on_cooc_stat.entry(cooc)
                .or_insert(CoocStats {
                    word_frequency: 0u8,
                    set_frequency: 0u32,
                    term_frequency: env.get_inverted_idx(&cooc).len() 
                });
            if ! already_word_frequency_inced.contains(&cooc) {
                current_cooc_stat.word_frequency += 1;
                already_word_frequency_inced.insert(cooc);
            }
            current_cooc_stat.set_frequency += count;

        }

    }

    let mut coocs_on_cooc_stat: Vec<(&str, &CoocStats)> =
        coocs_on_cooc_stat.iter().map(
            |(word_nr, cooc_stats)| (env.dict.get_word(word_nr), cooc_stats)).collect();

    coocs_on_cooc_stat.sort_unstable_by(
        |(_, cooc_stats_a), (_, cooc_stats_b)| 
        cooc_stats_a.word_frequency.cmp(&cooc_stats_b.word_frequency));

    println!("Done collecting syntagmatic context {:?}", coocs_on_cooc_stat);

}
