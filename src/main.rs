use std::env;
use std::fs;
use std::collections::HashMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

type SentenceId = u32;
type WordNr = u32;

// consider Option instead of an artificial 'null'
const EMPTY_WORD: u32 = std::u32::MAX;

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

struct WPair {
    w1: WordNr,
    w2: WordNr,
    _fitness: i16
}

impl WPair {
    fn new(w1: WordNr, w2: WordNr) -> WPair {
        WPair {
           w1, w2,
           _fitness: 0i16 
        }
    }

    fn new_str(w1: &str, w2: &str, env: &Env) -> WPair {

        let w1 = env.dict.get(w1).expect("w1 not found in dict.");
        let w2 = env.dict.get(w2).expect("w2 not found in dict.");

        WPair::new(*w1, *w2)
    }

    fn println(&self, env: &Env) {

        println!("fitness: {}, w1: {}, w2: {}",
                 self._fitness,
                 if self.w1 == EMPTY_WORD { "empty" } else { &env.dict_vec[self.w1 as usize] },
                 if self.w2 == EMPTY_WORD { "empty" } else { &env.dict_vec[self.w2 as usize] });
    }
}

struct Pattern {
    prefix: WordNr,
    infix: Vec<WordNr>,
    suffix: WordNr,
    order: bool,
    fitness: i16
}

impl Clone for Pattern {
    fn clone(&self) -> Pattern {
        Pattern {
            prefix: self.prefix,
            infix: self.infix.clone(),
            suffix: self.suffix,
            order: self.order,
            fitness: self.fitness
        }
    }
}

impl Pattern {
    fn new(prefix: WordNr, infix: Vec<WordNr>, suffix: WordNr, order: bool) -> Pattern {
        Pattern {
            prefix, infix, suffix, order,
            fitness: 9i16
        }
    }

    fn println(&self, env: &Env) {

        println!("fitness: {}, prefix: {}, infix: {:?}, suffix: {}, order: {}",
                 self.fitness,
                 if self.prefix == EMPTY_WORD { "empty" } else { &env.dict_vec[self.prefix as usize] },
                 {self.infix.iter().map(|word_nr| &env.dict_vec[*word_nr as usize]).collect::<Vec<&String>>()},
                 if self.suffix == EMPTY_WORD { "empty" } else { &env.dict_vec[self.suffix as usize] },
                 self.order);
    }
}

struct Env {
    dict_vec: Vec<String>,
    dict: HashMap<String, WordNr>,
    inverted_idx: HashMap<WordNr, HashSet<SentenceId>>,
    sentences: Vec<Vec<WordNr>>,
    _pairs: Vec<WPair>
}

impl Env {
    fn new() -> Env {
        Env {
            dict_vec: Vec::new(),
            dict: HashMap::new(),
            inverted_idx: HashMap::new(),
            sentences: Vec::new(), 
            _pairs: Vec::new() 
        }
    }

    fn add_word(&mut self, w: &str) -> WordNr {
        if self.dict.contains_key(w) {
            return self.dict[w];
        } else {
            let i = self.dict_vec.len() as WordNr; 

            //TODO rly two copies needed?
            self.dict_vec.push(w.to_owned());
            self.dict.insert(w.to_owned(), i);
            return i;
        }
    }

    fn add_inv_idx(&mut self, w: WordNr, s_id: SentenceId) {
        self.inverted_idx.entry(w)
            .or_insert(HashSet::new())
            .insert(s_id);
    }

}
    
    
fn read_xml_file(file_name: &str, env: &mut Env){

    let mut reader = Reader::from_file(file_name)
        .expect("Could not read from input file.");

    let mut buf = Vec::new();

    let mut read: bool = false;

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
                        read = false;
                    }, 
                    _ => (),
                }
            }
           
            Ok(Event::Text(ref e)) if read => {

                let s: String = e.unescape_and_decode(&reader)
                   .expect("Error while reading text from xml.");
                
                let mut sentences = s.unicode_sentences()
                    .map(|sent| sent
                         .split_word_bounds()
                         .filter(|word| *word != " ")
                         .map(|word| env.add_word(word))
                         .collect::<Vec<u32>>())
                    .collect::<Vec<Vec<u32>>>();

                for (i, sent) in sentences.iter().enumerate() {
                    let sentence_id: SentenceId = (i + env.sentences.len()) as u32; 
                    for word in sent {
                        env.add_inv_idx(*word, sentence_id);
                    }
                }

                env.sentences.append(&mut sentences);
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
    let idx_w1 = env.inverted_idx.get(&wpair.w1).expect("w1 not found in inverted index");
    let idx_w2 = env.inverted_idx.get(&wpair.w2).expect("w2 not found in inverted index");

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
    let sentence_ids_infix_pos_0 = env.inverted_idx.get(&pattern.infix[0])
        .expect("infix word not found in inverted index");

    let mut sentence_ids: HashSet<SentenceId> = sentence_ids_infix_pos_0.to_owned();

    for i in 1..l {
        let sentence_ids_infix_pos_i = env.inverted_idx.get(&pattern.infix[i])
            .expect("infix word not found in inverted index");

        sentence_ids = sentence_ids.intersection(sentence_ids_infix_pos_i)
            .map(|s_id| *s_id)
            .collect::<HashSet<SentenceId>>(); 
    }

    // now search every sentence for the first infix word and look 
    // the next infix.len() - 2 words.

    sentence_ids.iter()
        .map(|s_id| {
            let sent = &env.sentences[*s_id as usize];
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

            (infix_pos_0_idx, sent)

        })
        .filter(|(infix_pos_0_idx, sent)| {

            for i in 1..pattern.infix.len() {
                let p = infix_pos_0_idx + i;
                if sent.len() == p || sent[p] != pattern.infix[i] {
                    return false;
                }
            }

            true
            
        })
        .map(|(infix_pos_0_idx, sent)| {
            let w1 = if infix_pos_0_idx == 0 {
                EMPTY_WORD
            } else {
                sent[infix_pos_0_idx - 1]
            };

            let idx2 = infix_pos_0_idx + pattern.infix.len();
            let w2 = if idx2 == sent.len() {
                EMPTY_WORD
            } else {
                sent[idx2]
            };

            if pattern.order {
                WPair::new(w1, w2)
            } else {
                WPair::new(w2, w1)
            }
        })
        .filter(|WPair {w1, w2, _fitness}|
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

fn _translate <'a> (sent: &Vec<WordNr>, env: &'a Env) -> Vec<&'a String>{
    sent.iter().map(|word_nr| &env.dict_vec[*word_nr as usize]).collect()
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


fn main() {

    println!("starting.");
    let mut env = Env::new();

    let args: Vec<String> = env::args().collect(); 
    if args.len() < 1 {
        panic!("please provide xml directory as parameter.");
    }

    println!("reading files from directory {}.", &args[1]);
    for file_name in file_names_from_directory(&args[1])
        .expect("could not read input directory.") {

        read_xml_file(&file_name, &mut env);
    }

    println!("done reading files from directory.");

    println!("{} sentences loaded, with {} distinct words."
             , env.sentences.len(), env.dict_vec.len()); 

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
                    let sent = &env.sentences[*s_id as usize];
                    extract_pattern(wpair, sent)
                }).collect::<Vec<Pattern>>();

            (wpair, patterns)
        }).collect(); 
    println!("done finding matches for input wpairs.");
    
    println!("qualifying found matches to patterns.");
    let mut pattern_cache: HashMap<Vec<WordNr>, Pattern> = HashMap::new();

    let mut pattern_count = 0;
    for (_wpair, patterns) in wpair_on_patterns {
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
            // to the underlying relation.
            p.fitness += PATTERN_WPAIR_BOOST;

            // boost for every time the pattern / infix
            // was found -> intuition: pattern is quite,
            // often seen. Could indicate that
            // pattern is overly general - minor malus.
            p.fitness += PATTERN_PATTERN_BOOST;
                
        }
    }
    println!("done qualifying found matches to patterns."); 
    println!("pattern count: {}", pattern_count);

    let mut patterns: Vec<&Pattern> = pattern_cache.values().collect();

    println!("sorting patterns by fitness.");
    patterns.sort_unstable_by(
        |a, b| i16::cmp(&a.fitness, &b.fitness).reverse());
    println!("done sorting patterns by fitness.");

    println!("Top 10 final patterns with extracted wpairs:");
    for pattern in patterns.iter().take(10) {
        pattern.println(&env);

        let wpairs = find_matches_pattern(&pattern, &env);

        for wpair in wpairs.iter().take(10) {
            wpair.println(&env);
        }

    }
    

    
    
    // for (wpair, patterns) in wpair_on_patterns {
    //     println!("patterns for wpair: (\"{}\", \"{}\") {}",
    //              &env.dict_vec[wpair.w1 as usize],
    //              &env.dict_vec[wpair.w2 as usize],
    //              patterns.len());

        
    //     // for pattern in patterns {
    //     //     println!("prefix: {}, infix: {:?}, suffix: {}, order: {}",
    //     //              if pattern.prefix == EMPTY_WORD { "empty" } else { &env.dict_vec[pattern.prefix as usize] },
    //     //              {pattern.infix.iter().map(|word_nr| &env.dict_vec[*word_nr as usize]).collect::<Vec<&String>>()},
    //     //              if pattern.suffix == EMPTY_WORD { "empty" } else { &env.dict_vec[pattern.suffix as usize] },
    //     //              pattern.order);
    //     // }
    // }

    // for (wpair, s_ids) in wpair_on_sentence_ids {

    //     println!("sentences for wpair: (\"{}\", \"{}\")",
    //              &env.dict_vec[wpair.w1 as usize],
    //              &env.dict_vec[wpair.w2 as usize]);

    //     for s_id in s_ids {
    //         let sent = &env.sentences[s_id as usize];
    //         println!("{:?}", translate(sent, &env));
    //     }

    // }

}
