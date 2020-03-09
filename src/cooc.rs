use super::types::{AsyncLogger, DipreInput, EMPTY_WORD,
WordNr, SentenceId, Env, WPair, Pattern}; 


fn cooccurrences_for_word(word: &WordNr, env: &Env) {
    let sentences = env.inverted_idx.inverted_idx.get(word)
        .expect("word not found in inverted index");

}

fn do_cooc(set: Vec<String>, env: &Env) {


    




}
