use super::types::{AsyncLogger, DipreInput, EMPTY_WORD,
WordNr, SentenceId, Env, WPair, Pattern}; 

use log::{error, warn};
use std::collections::HashMap;
use std::collections::HashSet;

// pattern was found for one or more wpairs 
const PATTERN_WPAIR_BOOST: i16 = 10;
// pattern (infix) was found one or more times
const PATTERN_PATTERN_BOOST: i16 = -2;
// pattern is to short MALUS
const PATTERN_SHORT_SIZED_BOOST: i16 = -40;
// pattern is medium sized
const PATTERN_MEDIUM_SIZED_BOOST: i16 = 0;
// pattern is to long MALUS
const PATTERN_LONG_SIZED_BOOST: i16 = -40;

const PATTERN_SURVIVOR_THRESHOLD: i16 = 12;

const WPAIR_SURVIVOR_THRESHOLD: i16 = 20;

// word appears frequently in the global corpus
// needs to be dependent on the size of the corpus
const WPAIR_WORD_GLOBAL_FREQUENCY_BOOST_PER_SENTENCE: f32 = -0.1; 

// wpair is identified over various patterns
const WPAIR_PATTERN_BOOST: i16 = 10;

fn find_matches_wpair(wpair: &WPair, env: &Env) -> HashSet<SentenceId>{
    let idx_w1 = env.inverted_idx.inverted_idx.get(&wpair.w1)
        .expect("w1 not found in inverted index");
    let idx_w2 = env.inverted_idx.inverted_idx.get(&wpair.w2)
        .expect("w2 not found in inverted index");

    idx_w1.intersection(&idx_w2)
        .copied()
        .collect::<HashSet<SentenceId>>()
}

// fn find_matches_pattern <'a> (pattern: &Pattern, env: &'a Env) -> Vec<& 'a Vec<WordNr>> {
fn find_matches_pattern(pattern: &Pattern, env: &Env) -> Vec<WPair> {

    let l = pattern.infix.len();

    // for empty infixes don't do anything
    if l < 1 {
        return Vec::new();
    }

    // a very naive approach just combines all inverted indizes
    // to reduce search space by intersecting single infix words

    // another approach would be more memory intense, by not only
    // storing sentence_id for every word occurrence but also store
    // position in sentence. With that we could find matching sentences
    // without even look at a single sentence, just by comparing
    // occurrence position - problem multiple occurrences in single sent. 


    // take sentence_ids for first word of infix
    let sentence_ids_infix_pos_0 = env.inverted_idx.inverted_idx
        .get(&pattern.infix[0])
        .expect("infix word not found in inverted index");

    let mut sentence_ids: HashSet<SentenceId> = sentence_ids_infix_pos_0.to_owned();

    for i in 1..l {
        let sentence_ids_infix_pos_i = env.inverted_idx.inverted_idx
            .get(&pattern.infix[i])
            .expect("infix word not found in inverted index");

        sentence_ids = sentence_ids
            .intersection(sentence_ids_infix_pos_i)
            .copied()
            .collect::<HashSet<SentenceId>>(); 
    }

    // now search every sentence for the first infix word and look 
    // the next infix.len() - 2 words.

    sentence_ids.iter()
        .map(|s_id| {
            let sent = &env.sentences.sentences[*s_id as usize];
            let mut infix_pos_0_idx = std::usize::MAX;

            for (i, word) in sent.iter().enumerate() {
                if *word == pattern.infix[0] {
                    infix_pos_0_idx = i; 
                    break;
                }
            }

            if infix_pos_0_idx == std::usize::MAX {
                panic!("find_matches_pattern: Could not find word {} in {:?}.",
                       pattern.infix[0], sent);
            }

            (infix_pos_0_idx, sent, s_id)

        })
    .filter(|(infix_pos_0_idx, sent, _s_id)| {

        for i in 1..pattern.infix.len() {
            let p = infix_pos_0_idx + i;
            if sent.len() == p || sent[p] != pattern.infix[i] {
                return false;
            }
        }

        true

    })
    .map(|(infix_pos_0_idx, sent, s_id)| {
        let w1 = if infix_pos_0_idx == 0 {
            EMPTY_WORD
        } else {
            sent[infix_pos_0_idx - 1]
        };

        let idx2 = infix_pos_0_idx + pattern.infix.len();
        let w2 = if idx2 == sent.len() {
            EMPTY_WORD
        } else {
            // special case THE
            if sent[idx2] == env.the {
                if idx2 + 1 == sent.len() {
                    // there is "the" as the final word of a sentence?
                    error!("Something strange in my neighbourhood! Call Ghost Busters!");
                    error!("theres a sentence which ends with \"the\"! Let's take a look.");
                    error!("{:?}", translate(&env.sentences.sentences[*s_id as usize], &env));
                    for i in 1..4 {
                        error!("{:?}", translate(&env.sentences.sentences[(s_id + i) as usize], &env));
                    }
                    EMPTY_WORD
                } else {
                    sent[idx2 + 1]
                }
            } else {
                sent[idx2]
            }
        };

        if pattern.order {
            WPair::new(w1, w2)
        } else {
            WPair::new(w2, w1)
        }
    })
    .filter(|WPair {w1, w2, ..}|
            !(*w1 == EMPTY_WORD || *w2 == EMPTY_WORD) )
        .collect()
}

fn extract_pattern(wpair: &WPair, sent: &[WordNr]) -> Pattern {

    let mut idx1 = std::usize::MAX;
    let mut idx2 = std::usize::MAX;
    for (i, w) in sent.iter().enumerate() {
        if *w == wpair.w1 {
            idx1 = i;
        } else if *w == wpair.w2 {
            idx2 = i;
        }
    }

    if idx1 == std::usize::MAX && idx2 == std::usize::MAX {
        panic!("Either w1 {} or w2 {} not found in {:?}", wpair.w1, wpair.w2, sent);
    }

    let (idx1, idx2, order) = if idx1 < idx2 {
        (idx1, idx2, true)
    } else {
        (idx2, idx1, false)
    };

    let prefix = if idx1 == 0 { EMPTY_WORD } else { sent[idx1 - 1] };

    let suffix = if idx2 < sent.len() - 1 {
        sent[idx2 + 1]
    } else { EMPTY_WORD }; 

    Pattern::new(prefix, sent[idx1 + 1..idx2].to_vec(), suffix, order)

}

fn translate <'a> (sent: &[WordNr], env: &'a Env) -> Vec<&'a str>{
    sent.iter().map(|word_nr| env.dict.get_word(word_nr)).collect()
}

pub async fn do_relex<F: AsyncLogger>(
    di: DipreInput, env: &Env, mut log: F) {

    let wpairs: Vec<WPair> = di.pairs.iter()
        .map(|(w1, w2)| WPair::new_str(w1, w2, env)).collect(); 

    log.log(format!("finding matches for input {:?} wpairs.", wpairs)).await;

    let wpair_on_patterns: Vec<(&WPair, Vec<Pattern>)> =
        wpairs.iter()
        .map(|wpair| {
            let sentence_ids = find_matches_wpair(wpair, &env);

            let patterns = sentence_ids.iter()
                .map(|s_id| {
                    let sent = &env.sentences.sentences[*s_id as usize];
                    extract_pattern(wpair, sent)
                }).collect::<Vec<Pattern>>();

            (wpair, patterns)
        }).collect(); 
    log.log(format!("done finding matches for input wpairs.")).await;

    log.log(format!("qualifying found matches to patterns.")).await;
    let mut pattern_cache: HashMap<Vec<WordNr>, Pattern> = HashMap::new();

    let mut pattern_count = 0;
    for (_wpair, patterns) in wpair_on_patterns {

        let mut already_wpair_boosted: HashSet<Vec<WordNr>> = HashSet::new();

        for pattern in patterns {

            pattern_count += 1;

            // TODO could be possibly memory leak, since
            // copy is created for every pattern, even if the
            // reference pattern exists in the cache

            let mut p_ = (&pattern).clone();

            let mut p = pattern_cache.entry(pattern.infix)
                .or_insert({
                    let infix_len = p_.infix.len();
                    if infix_len <= 1 {
                        p_.fitness += PATTERN_SHORT_SIZED_BOOST;
                    }
                    if infix_len > 1 && infix_len < 5 {
                        p_.fitness += PATTERN_MEDIUM_SIZED_BOOST;
                    }
                    if infix_len >= 5 {
                        p_.fitness += PATTERN_LONG_SIZED_BOOST;
                    }
                    p_
                });

            if p.prefix != pattern.prefix {
                p.prefix = EMPTY_WORD;
            }

            if p.suffix != pattern.suffix {
                p.suffix = EMPTY_WORD;
            }

            // boost for every wpair the pattern occured
            // -> intuition: pattern is able to identify a
            // more general range of wpairs - thus more suited
            // to the underlying relation. But boost only once
            // per wpair!
            if ! already_wpair_boosted.contains(&p.infix){
                p.fitness += PATTERN_WPAIR_BOOST;
                // TODO try to find solution with pointer
                // this is time consuming mem copy
                already_wpair_boosted.insert(p.infix.clone());
            }

            // boost for every time the pattern / infix
            // was found -> intuition: pattern is quite,
            // often seen. Could indicate that
            // pattern is overly general - minor malus.
            p.fitness += PATTERN_PATTERN_BOOST;

        }
    }
    log.log(format!("done qualifying found matches to patterns.")).await; 
    log.log(format!("pattern count: {}", pattern_count)).await;

    let patterns: Vec<&Pattern> = pattern_cache.values()
        .filter(|pattern| pattern.fitness >= PATTERN_SURVIVOR_THRESHOLD)
        .collect();
    log.log(format!("{} patterns left after applying threshold fitness of {}.",
             patterns.len(), PATTERN_SURVIVOR_THRESHOLD)).await;

    for pattern in &patterns {
        pattern.println(&env);
    }

    log.log(format!("finding new wpairs for surviving patterns (fitness >= {}).",
    PATTERN_SURVIVOR_THRESHOLD)).await;

    let pattern_on_wpairs = patterns.iter()
        .map(|pattern| {
            let wpairs = find_matches_pattern(&pattern, &env);
            (pattern, wpairs)
        });
    log.log(format!("done finding new wpairs for surviving patterns.")).await;

    log.log(format!("qualifying found wpairs.")).await;
    let mut wpair_cache: HashMap<(WordNr, WordNr), WPair> = HashMap::new();

    let wpair_word_frequency_boost =
        WPAIR_WORD_GLOBAL_FREQUENCY_BOOST_PER_SENTENCE / env.sentences.sentences.len() as f32;  

    log.log(format!("wpair_word_frequency_boost = {}", wpair_word_frequency_boost)).await;

    let mut wpair_count = 0;

    log.log(format!("qualifying found wpairs")).await;

    for (_pattern, wpairs) in pattern_on_wpairs {

        let mut already_pattern_boosted: HashSet<(WordNr, WordNr)> = HashSet::new();

        for wpair in wpairs {

            wpair_count += 1;

            let mut wp_ = wpair.clone();

            let wp = wpair_cache.entry((wpair.w1, wpair.w2))
                .or_insert({
                    // maybe the most dificult part to boost is
                    // the global term frequency. Intuition here
                    // is that if a wpair contains a very frequent term
                    // e.g. "the", then the resulting set of the inverted
                    // index will be huge in size, and we'll be punishing
                    // this in relation to the overall corpus size, since
                    // this term seem to be overly general

                    let calc_freq_boost = |w| {
                        env.inverted_idx.inverted_idx.get(w)
                            .expect("w not found in inverted index")
                            .len() as f32 * wpair_word_frequency_boost 
                    };

                    let w1_freq_boost = calc_freq_boost(&wp_.w1); 
                    let w2_freq_boost = calc_freq_boost(&wp_.w2); 

                    // this can get seriously wrong if the numbers outgrow
                    // i16::MIN, but if this happens our fitness score
                    // is messed up anyways
                    let mut save_cast = |wn_freq_boost, w| {
                        if wn_freq_boost < std::i16::MIN as f32 {
                            warn!("Word frequency boost outmaxed by {}",
                                     env.dict.get_word(w));
                            std::i16::MIN
                        } else {
                            // save cast now
                            wn_freq_boost as i16
                        }
                    };

                    let w1_freq_boost = save_cast(w1_freq_boost, &wp_.w1);
                    let w2_freq_boost = save_cast(w2_freq_boost, &wp_.w2);

                    wp_.fitness += w1_freq_boost + w2_freq_boost; 

                    wp_
                });


            // boost positively if a single wpair is defined
            // by more than one pattern.

            let tmp_wpair = (wp.w1, wp.w2);
            if ! already_pattern_boosted.contains(&tmp_wpair){
                wp.fitness += WPAIR_PATTERN_BOOST;
                // TODO try to find solution with pointer
                // this is time consuming mem copy
                already_pattern_boosted.insert(tmp_wpair);
            }

        }
    }

    log.log(format!("done qualifying found wpairs.")).await; 
    log.log(format!("wpair count: {}", wpair_count)).await;

    let mut wpairs: Vec<&WPair> = wpair_cache.values()
        .filter(|wpair| wpair.fitness >= WPAIR_SURVIVOR_THRESHOLD)
        .collect();
    log.log(format!("{} wpairs left after applying threshold fitness of {}.",
             wpairs.len(), WPAIR_SURVIVOR_THRESHOLD)).await;

    log.log(format!("sorting wpairs by fitness.")).await;
    wpairs.sort_unstable_by(
        |a, b| i16::cmp(&a.fitness, &b.fitness).reverse());
    log.log(format!("done sorting wpairs by fitness.")).await;

    log.log(format!("building a map of w1 to vec w2")).await;

    let mut w1_on_w2s: HashMap<&WordNr, Vec<&WordNr>> = HashMap::new();
    for wpair in wpairs {
        w1_on_w2s.entry(&wpair.w1)
            .or_insert_with(Vec::new)
            .push(&wpair.w2);
    }

    log.log(format!("done building a map of w1 to vec w2")).await;

    for (w1, w2s) in w1_on_w2s {
        log.log(format!("\"{}\":", env.dict.get_word(w1))).await;
        for w in w2s.iter().map(|w| env.dict.get_word(w)) {
            log.log(format!("\t \"{}\"", w)).await;
        }
    }
}


