use super::types::{CoocInput, WordNr, CoocFst, CoocSnd, Env}; 

use log::{info, warn};
use std::collections::HashMap;
use std::collections::HashSet;

// how many bootstrap words share this cooc
const COOC1_WORD_FREQUENCY_BOOST: isize = 50;
// how frequent is this cooc overall w.r.t bootstrap set
const COOC1_SET_FREQUENCY_BOOST: isize = 0;
// overall termfrequency i.e. how many sentences contain this term
// const COOC1_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE: f32 = -100.0;
const COOC1_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE: isize = -1;

const COOC1_SURVIVOR_THRESHOLD: isize = 100;

// how many cooc1 do cooccurr with that cooc2?
const COOC2_COOC1_FREQUENCY_BOOST: isize = 50;

// how frequent is this cooc2 in the whole cooc1 set
const COOC2_SET_FREQUENCY_BOOST: isize = 0;

// overall termfrequency i.e. how many sentences contain this term
const COOC2_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE: isize = -1;

const COOC2_SURVIVOR_THRESHOLD: isize = 100;

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

fn cooccurrences_for_word(word: WordNr, env: &Env) -> HashMap<WordNr, isize>{
    
    // get all sentences which contain word
    let sentence_ids = env.get_inverted_idx(&word);

    let mut word_on_count: HashMap<WordNr, isize> = HashMap::new(); 

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
                warn!("Word \"{}\" not found in dictionary 
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

pub fn do_conex(cooc_input: CoocInput, env: &Env) {

    // this can get seriously wrong if the numbers outgrow
    // i16::MIN, but if this happens our fitness score
    // is messed up anyways
    // let save_cast = |wn_freq_boost: f32, w| {
    //     if wn_freq_boost < std::i16::MIN as f32 {
    //         println!("Word frequency boost outmaxed by {}",
    //                         env.dict.get_word(&w));
    //         std::i16::MIN
    //     } else {
    //         // save cast now
    //         wn_freq_boost as i16
    //     }
    // };

    info!("Converting input {:?} into set of word numbers", cooc_input);
    let bootstrap_set = cooc_input_to_word_nr_set(&cooc_input, &env);
    info!("Done converting input into set of word numbers: {:?}",           
             bootstrap_set);

    // let wpair_word_frequency_boost =
    //     COOC1_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE / env.sentences.sentences.len() as f32;  

    // info!("wpair_word_frequency_boost = {}", wpair_word_frequency_boost);

    let mut coocs_on_cooc_fst: HashMap<WordNr, CoocFst> = HashMap::new(); 

    info!("Collecting syntagmatic context");
    for word in bootstrap_set {
        let coocs_for_word = cooccurrences_for_word(word, env);
        let mut already_word_frequency_boosted: HashSet<WordNr> = HashSet::new(); 

        for (cooc, count) in coocs_for_word {
            let mut cooc_fst = coocs_on_cooc_fst.entry(cooc)
                .or_insert({
                     let freq_boost = env.get_inverted_idx(&cooc).len() as isize
                         * COOC1_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE; 
                    // let freq_boost = env.get_inverted_idx(&cooc).len() as f32 
                    //     * wpair_word_frequency_boost;
                    // let freq_boost = save_cast(freq_boost, cooc);
                    CoocFst::new(cooc,freq_boost)});
                
            if ! already_word_frequency_boosted.contains(&cooc) {
                cooc_fst.fitness += COOC1_WORD_FREQUENCY_BOOST;
                already_word_frequency_boosted.insert(cooc);
            }

            cooc_fst.fitness += count * COOC1_SET_FREQUENCY_BOOST;
        }

    }

    // let mut coocs_on_cooc_fst: Vec<(&str, &CoocFst)> =
    //     coocs_on_cooc_fst.iter().map(
    //         |(word_nr, cooc_fst)| (env.dict.get_word(word_nr), cooc_fst)).collect();

    // coocs_on_cooc_fst.sort_unstable_by(
    //     |(_, cooc_fst_a), (_, cooc_fst_b)| 
    //     cooc_fst_a.fitness.cmp(&cooc_fst_b.fitness));

    // info!("Done collecting syntagmatic context {:?}", coocs_on_cooc_fst);
    info!("Done collecting syntagmatic context.");
    
    let l1 = coocs_on_cooc_fst.len();

    // filter by COOC1_FITNESS_THRESHOLD
    let mut cooc_fsts: Vec<CoocFst> = coocs_on_cooc_fst.iter()
        .filter(|(_, c)| c.fitness >= COOC1_SURVIVOR_THRESHOLD)
        .map(|(_, c)| (*c).clone())
        .collect();

    info!("{} from {} syntagmatic coocs left after applying threshold of {}",
        cooc_fsts.len(), l1, COOC1_SURVIVOR_THRESHOLD); 

    // cooc_fsts.sort_unstable_by(
    //     |a, b| 
    //     a.fitness.cmp(&b.fitness));

    // info!("{:?}", cooc_fsts.iter().map(|c| 
    //         (env.dict.get_word(&c.word), c.fitness)).collect::<Vec<(&str, isize)>>());
    
    let mut coocs_on_cooc_snd: HashMap<WordNr, CoocSnd> = HashMap::new(); 

    info!("Collecting paradigmatic context");
    for cooc in cooc_fsts {
        let coocs_for_word = cooccurrences_for_word(cooc.word, env);
        let mut already_cooc_frequency_boosted: HashSet<WordNr> = HashSet::new(); 

        for (cooc, count) in coocs_for_word {
            let mut cooc_snd = coocs_on_cooc_snd.entry(cooc)
                .or_insert({
                     let freq_boost = env.get_inverted_idx(&cooc).len() as isize
                         * COOC2_GLOBAL_TERM_FREQUENCY_BOOST_PER_SENTENCE; 
                    // let freq_boost = env.get_inverted_idx(&cooc).len() as f32 
                    //     * wpair_word_frequency_boost;
                    // let freq_boost = save_cast(freq_boost, cooc);
                    CoocSnd::new(cooc,freq_boost)});
                
            if ! already_cooc_frequency_boosted.contains(&cooc) {
                cooc_snd.fitness += COOC2_COOC1_FREQUENCY_BOOST;
                already_cooc_frequency_boosted.insert(cooc);
            }

            cooc_snd.fitness += count * COOC2_SET_FREQUENCY_BOOST;
        }

    }
    info!("Done collecting paradigmatic context.");
   
    let l2 = coocs_on_cooc_snd.len();
    
    // filter by COOC2_FITNESS_THRESHOLD
    let mut cooc_snds: Vec<CoocSnd> = coocs_on_cooc_snd.iter()
        .filter(|(_, c)| c.fitness >= COOC2_SURVIVOR_THRESHOLD)
        .map(|(_, c)| (*c).clone())
        .collect();

    info!("{} from {} paradigmatic coocs left after applying threshold of {}",
        cooc_snds.len(), l2, COOC2_SURVIVOR_THRESHOLD); 

    cooc_snds.sort_unstable_by(
        |a, b| 
        a.fitness.cmp(&b.fitness));

    info!("{:?}", cooc_snds.iter().map(|c| 
            (env.dict.get_word(&c.word), c.fitness)).collect::<Vec<(&str, isize)>>());
}
