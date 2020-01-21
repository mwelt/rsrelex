mod types;

use types::{EMPTY_WORD, WordNr, SentenceId, Env, WPair, Pattern};

use std::env;
use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

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

fn read_xml_file(file_name: &str, env: &mut Env){

    let mut reader = Reader::from_file(file_name)
        .expect("Could not read from input file.");

    let mut buf = Vec::new();

    let mut read: bool = false;

    let mut curr_str = String::new();

    println!("Start reading file {}", file_name);

    loop {
        match reader.read_event(&mut buf) {

            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"AbstractText" => {
                        read = true;
                    }, 
                    _ => (),
                }
            },

            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"AbstractText" => {

                        let mut sentences = curr_str.unicode_sentences()
                            .map(|sent| sent
                                 .split_word_bounds()
                                 .filter(|word| *word != " ")
                                 .map(|word| env.add_word(word))
                                 .collect::<Vec<u32>>())
                            .collect::<Vec<Vec<u32>>>();

                        for (i, sent) in sentences.iter().enumerate() {
                            let sentence_id: SentenceId = (i + env.sentences.sentences.len()) as u32; 
                            for word in sent {
                                env.add_inv_idx(*word, sentence_id);
                            }
                        }

                        env.sentences.sentences.append(&mut sentences);

                        curr_str = String::new(); 
                        read = false;
                    }, 
                    _ => (),
                }
            }
           
            Ok(Event::Text(ref e)) if read => {

                let s: String = e.unescape_and_decode(&reader)
                   .expect("Error while reading text from xml.");

                curr_str.push_str(&s);
            },

            Err(e) => panic!(
                "Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }

    println!("done reading file.");
}

fn find_matches_wpair(wpair: &WPair, env: &Env) -> HashSet<SentenceId>{
    let idx_w1 = env.inverted_idx.inverted_idx.get(&wpair.w1)
        .expect("w1 not found in inverted index");
    let idx_w2 = env.inverted_idx.inverted_idx.get(&wpair.w2)
        .expect("w2 not found in inverted index");

    idx_w1.intersection(&idx_w2)
        .map(|s_id| *s_id)
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

        sentence_ids = sentence_ids.intersection(sentence_ids_infix_pos_i)
            .map(|s_id| *s_id)
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
                        println!("Somthing strange in my neighbourhood! Call Ghost Busters!");
                        println!("theres a sentence which ends with \"the\"! Let's take a look.");
                        println!("{:?}", translate(&env.sentences.sentences[*s_id as usize], &env));
                        for i in 1..4 {
                            println!("{:?}", translate(&env.sentences.sentences[(s_id + i) as usize], &env));
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
        .filter(|WPair {w1, w2, fitness: _}|
                if *w1 == EMPTY_WORD || *w2 == EMPTY_WORD {
                    false
                } else {
                    true
                })
        .collect()
}

fn extract_pattern(wpair: &WPair, sent: &Vec<WordNr>) -> Pattern {

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

fn translate <'a> (sent: &Vec<WordNr>, env: &'a Env) -> Vec<&'a str>{
    sent.iter().map(|word_nr| env.dict.get_word(word_nr)).collect()
}

fn file_names_from_directory(dir: &str) -> std::io::Result<Vec<String>> {
    let mut r = Vec::new();
    for elem in fs::read_dir(dir)? {
        let p = elem?.path();
        if ! p.is_dir() {
            r.push(p.to_str().unwrap().to_owned());
        }
    }
    Ok(r)
}


fn read_and_serialize_xmls(args: &Vec<String>){
    println!("starting read_and_serialize_xmls.");
    let mut env = Env::new();

    println!("reading files from directory {}.", &args[1]);
    for file_name in file_names_from_directory(&args[1])
        .expect("could not read input directory.") {

        read_xml_file(&file_name, &mut env);
    }

    println!("done reading files from directory.");

    println!("{} sentences loaded, with {} distinct words."
             , env.sentences.sentences.len(), env.dict.dict_vec.len()); 

    // TODO save sentences, dictionary and inverted index seperately
    // println!("starting writing binary file {}.", DATA_BIN);

    // let mut f = BufWriter::new(File::create(DATA_BIN).unwrap());
    // serialize_into(&mut f, &env.data).unwrap();

    // println!("done writing binary file.");
    
    // println!("done read_and_serialize_xmls.");
}

fn main() {

    let args: Vec<String> = env::args().collect(); 

    if args.len() > 1 {
        read_and_serialize_xmls(&args);
        return;
    }

    println!("starting.");
    let mut env = Env::new();

    // TODO load sentences, dictionary and inverted index sperately
    // println!("start reading binary data from {}.", DATA_BIN);
    // let mut f = BufReader::new(File::open(DATA_BIN).unwrap());
    // env.data = deserialize_from(&mut f).unwrap();
    // println!("done reading binary data.");

    println!("{} sentences loaded, with {} distinct words."
             , env.sentences.sentences.len(),
             env.dict.dict_vec.len()); 

    env.the = env.dict.get_opt_nr("the")
        .expect("\"the\" not found in dictionary.");

    let wpairs = vec![
        WPair::new_str("organs", "liver", &env),
        WPair::new_str("organs", "lung", &env),
        WPair::new_str("bacteria", "Staphylococcus", &env),
        WPair::new_str("bacteria", "Streptococcus", &env),
        WPair::new_str("organs", "esophagus", &env)
        // WPair::new_str("cancer", "BRCA1", &env),
        // WPair::new_str("cancer", "UV", &env),
        // WPair::new_str("cancer", "ultraviolet", &env),
        // WPair::new_str("cancer", "alcohol", &env),
        // WPair::new_str("cancer", "tobacco", &env),
    ];

    println!("finding matches for input {} wpairs.", wpairs.len());
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
    println!("done finding matches for input wpairs.");
    
    println!("qualifying found matches to patterns.");
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
    println!("done qualifying found matches to patterns."); 
    println!("pattern count: {}", pattern_count);

    let patterns: Vec<&Pattern> = pattern_cache.values()
        .filter(|pattern| pattern.fitness >= PATTERN_SURVIVOR_THRESHOLD)
        .collect();
    println!("{} patterns left after applying threshold fitness of {}.",
             patterns.len(), PATTERN_SURVIVOR_THRESHOLD);

    for pattern in &patterns {
        pattern.println(&env);
    }

    println!("finding new wpairs for surviving patterns (fitness >= {}).",
             PATTERN_SURVIVOR_THRESHOLD);
   
    let pattern_on_wpairs = patterns.iter()
        .map(|pattern| {
            let wpairs = find_matches_pattern(&pattern, &env);
            (pattern, wpairs)
        });
    println!("done finding new wpairs for surviving patterns.");

    println!("qualifying found wpairs.");
    let mut wpair_cache: HashMap<(WordNr, WordNr), WPair> = HashMap::new();

    let wpair_word_frequency_boost =
        WPAIR_WORD_GLOBAL_FREQUENCY_BOOST_PER_SENTENCE / env.sentences.sentences.len() as f32;  

    println!("wpair_word_frequency_boost = {}", wpair_word_frequency_boost);

    let mut wpair_count = 0;

    println!("qualifying found wpairs");
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
                    let save_cast = |wn_freq_boost, w| {
                        if wn_freq_boost < std::i16::MIN as f32 {
                            println!("Word frequency boost outmaxed by {}",
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

    println!("done qualifying found wpairs."); 
    println!("wpair count: {}", wpair_count);

    let mut wpairs: Vec<&WPair> = wpair_cache.values()
        .filter(|wpair| wpair.fitness >= WPAIR_SURVIVOR_THRESHOLD)
        .collect();
    println!("{} wpairs left after applying threshold fitness of {}.",
             wpairs.len(), WPAIR_SURVIVOR_THRESHOLD);
    
    println!("sorting wpairs by fitness.");
    wpairs.sort_unstable_by(
        |a, b| i16::cmp(&a.fitness, &b.fitness).reverse());
    println!("done sorting wpairs by fitness.");

    println!("building a map of w1 to vec w2");

    let mut w1_on_w2s: HashMap<&WordNr, Vec<&WordNr>> = HashMap::new();
    for wpair in wpairs {
        w1_on_w2s.entry(&wpair.w1)
            .or_insert(Vec::new())
            .push(&wpair.w2);
    }
    println!("done building a map of w1 to vec w2");

    for (w1, w2s) in w1_on_w2s {
        println!("\"{}\":", env.dict.get_word(w1));
        for w in w2s.iter().map(|w| env.dict.get_word(w)) {
            println!("\t \"{}\"", w);
        }
    }

}
