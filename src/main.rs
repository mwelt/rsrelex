use std::collections::HashMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

type SentenceId = u32;
type WordNr = u32;

// consider Option instead of an artificial 'null'
const EMPTY_WORD: u32 = std::u32::MAX;
    
struct WPair {
    w1: WordNr,
    w2: WordNr,
    confidence: i16
}

impl WPair {
    fn new(w1: WordNr, w2: WordNr) -> WPair {
        WPair {
           w1, w2,
           confidence: 0i16 
        }
    }

    fn new_str(w1: &str, w2: &str, env: &Env) -> WPair {

        let w1 = env.dict.get(w1).expect("w1 not found in dict.");
        let w2 = env.dict.get(w2).expect("w2 not found in dict.");

        WPair::new(*w1, *w2)
    }
}

struct Pattern {
    prefix: WordNr,
    infix: Vec<WordNr>,
    suffix: WordNr,
    order: bool,
    confidence: i16
}

impl Pattern {
    fn new(prefix: WordNr, infix: Vec<WordNr>, suffix: WordNr, order: bool) -> Pattern {
        Pattern {
            prefix, infix, suffix, order,
            confidence: 9i16
        }
    }
}

struct Env {
    dict_vec: Vec<String>,
    dict: HashMap<String, WordNr>,
    inverted_idx: HashMap<WordNr, HashSet<SentenceId>>,
    sentences: Vec<Vec<WordNr>>,
    pairs: Vec<WPair>
}

impl Env {
    fn new() -> Env {
        Env {
            dict_vec: Vec::new(),
            dict: HashMap::new(),
            inverted_idx: HashMap::new(),
            sentences: Vec::new(), 
            pairs: Vec::new() 
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

    println!("Starting reading file {}", file_name);

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

fn find_matches(wpair: &WPair, env: &Env) -> HashSet<SentenceId>{
    let idx_w1 = env.inverted_idx.get(&wpair.w1).expect("w1 not found in inverted index");
    let idx_w2 = env.inverted_idx.get(&wpair.w2).expect("w2 not found in inverted index");

    idx_w1.intersection(&idx_w2)
        .map(|s_id| *s_id)
        .collect::<HashSet<SentenceId>>()
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

fn translate <'a> (sent: &Vec<WordNr>, env: &'a Env) -> Vec<&'a String>{
    sent.iter().map(|word_nr| &env.dict_vec[*word_nr as usize]).collect()
}

fn main() {

    let mut env = Env::new();

    let file_names = vec![
        // "data/pubmed19n0094.xml",
        // "data/pubmed19n0281.xml",
        // "data/pubmed19n0416.xml",
        // "data/pubmed19n0587.xml",
        // "data/pubmed19n0635.xml",
        // "data/pubmed19n0839.xml",
        // "data/pubmed19n0902.xml",
        // "data/pubmed19n0162.xml",
        // "data/pubmed19n0304.xml",
        // "data/pubmed19n0464.xml",
        // "data/pubmed19n0599.xml",
        // "data/pubmed19n0637.xml",
        // "data/pubmed19n0868.xml",
        // "data/pubmed19n0955.xml",
        // "data/pubmed19n0271.xml",
        // "data/pubmed19n0389.xml",
        // "data/pubmed19n0568.xml",
        // "data/pubmed19n0604.xml",
        // "data/pubmed19n0823.xml",
        "data/pubmed19n0879.xml", //--
        // "data2/pubmed19n0094.xml",
        // "data2/pubmed19n0162.xml",
        // "data2/pubmed19n0271.xml",
        // "data2/pubmed19n0281.xml",
        // "data2/pubmed19n0587.xml",
        // "data2/pubmed19n0902.xml"
    ];

    for file_name in file_names {
        read_xml_file(file_name, &mut env);
    }

    println!("size of sentences vec {}", env.sentences.len()); 
    println!("size of dict_vec {}", env.dict_vec.len()); 

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

    let wpair_on_patterns: Vec<(&WPair, Vec<Pattern>)> =
        wpairs.iter()
        .map(|wpair| {
            let sentence_ids = find_matches(wpair, &env);
            
            let patterns = sentence_ids.iter()
                .map(|s_id| {
                    let sent = &env.sentences[*s_id as usize];
                    extract_pattern(wpair, sent)
                }).collect::<Vec<Pattern>>();

            (wpair, patterns)
        }).collect(); 
    
    for (wpair, patterns) in wpair_on_patterns {
        println!("patterns for wpair: (\"{}\", \"{}\") {}",
                 &env.dict_vec[wpair.w1 as usize],
                 &env.dict_vec[wpair.w2 as usize],
                 patterns.len());

        
        // for pattern in patterns {
        //     println!("prefix: {}, infix: {:?}, suffix: {}, order: {}",
        //              if pattern.prefix == EMPTY_WORD { "empty" } else { &env.dict_vec[pattern.prefix as usize] },
        //              {pattern.infix.iter().map(|word_nr| &env.dict_vec[*word_nr as usize]).collect::<Vec<&String>>()},
        //              if pattern.suffix == EMPTY_WORD { "empty" } else { &env.dict_vec[pattern.suffix as usize] },
        //              pattern.order);
        // }
    }

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
