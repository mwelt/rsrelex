use std::collections::HashMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

type SentenceId = u32;
type WordNr = u32;

struct WPair {
    w1: WordNr,
    w2: WordNr,
    confidence: u8
}

impl WPair {
    fn new(w1: WordNr, w2: WordNr) -> WPair {
        WPair {
           w1, w2,
           confidence: 0u8 
        }
    }

    fn new_str(w1: &str, w2: &str, env: &Env) -> WPair {

        let w1 = env.dict.get(w1).expect("w1 not found in dict.");
        let w2 = env.dict.get(w2).expect("w2 not found in dict.");

        WPair::new(*w1, *w2)
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

fn find_matches(wpair: &mut WPair, env: &Env) -> HashSet<SentenceId>{
    let idx_w1 = env.inverted_idx.get(&wpair.w1).expect("w1 not found in inverted index");
    let idx_w2 = env.inverted_idx.get(&wpair.w2).expect("w1 not found in inverted index");

    idx_w1.intersection(&idx_w2)
        .map(|s_id| *s_id)
        .collect::<HashSet<SentenceId>>()
}



fn main() {

    let mut env = Env::new();

    // let file_name = "data2/pubmed19n0587.xml"; //"data2/pubmed19n0902.xml";
    let file_name = "data2/pubmed19n0902.xml";

    read_xml_file(file_name, &mut env);

    println!("size of sentences vec {}", env.sentences.len()); 
    println!("size of dict_vec {}", env.dict_vec.len()); 

    // println!("{:?}", sentences_orig[0]); 
    // println!("{:?}", sentences_translated[0]); 
    // println!("{:?}", sentences_translated[0].iter()
    //          .map(|word_nr| &dict_vec[*word_nr as usize])
    //          .collect::<Vec<&String>>());

    let sentences_with_and = env.dict.get("cancer")
        .and_then(|word_nr| env.inverted_idx.get(word_nr))
        .expect("No entry for word \"cancer\" found in dict or in inverted index.");

    println!("count sentences with word \"cancer\" {}", sentences_with_and.len());

    // println!("2nd of txt vec {:?}", txt[1]); 
    // println!("2nd of txt vec {:?}", txt[3]); 
    // println!("last of txt vec {:?}", txt[txt.len() - 1]);
}
