use super::types::{AsyncLogger, CoocInput, EMPTY_WORD,
WordNr, SentenceId, Env, WPair, Pattern}; 

use std::collections::HashMap;
use std::collections::HashSet;

fn cooccurrences_for_word(word: &WordNr, env: &Env) -> HashSet<WordNr>{
    
    // get all sentences which contain word
    let sentence_ids = env.get_inverted_idx(word);

    // flat map all cooccurrences into a set
    sentence_ids.iter().flat_map(|s_id| {
        env.get_sentence(s_id).iter().copied()
    }).collect::<HashSet<WordNr>>()

}

fn cooc_input_to_word_nr_set(cooc_input: &CoocInput, env: &Env) 
    -> HashSet<WordNr>{

    cooc_input.set.iter()
        .map(|word_str| env.dict.get_opt_nr(word_str)
             .unwrap_or_else(
                 || panic!("No word number found for word {}.", word_str)))
        .collect()
}

pub fn do_cooc(cooc_input: CoocInput, env: &Env) {

    println!("Converting input {:?} into set of word numbers", cooc_input);
    let bootstrap_set = cooc_input_to_word_nr_set(&cooc_input, &env);
    println!("Done converting input into set of word numbers: {:?}",           
             bootstrap_set);

    let mut coocs_on_count: HashMap<WordNr, u32> = HashMap::new(); 

    println!("Collecting syntagmatic context");
    for word in bootstrap_set {
        let coocs_for_word = cooccurrences_for_word(&word, env);

        for cooc in coocs_for_word {
            let current_count = coocs_on_count.entry(cooc)
                .or_insert(0);
            *current_count += 1;
        }

    }
    println!("Done collecting syntagmatic context {:?}",
             coocs_on_count.iter()
             .map(|(k, v)| {(env.dict.get_word(k), v)})
             .collect::<HashMap<&str, &u32>>());
}
