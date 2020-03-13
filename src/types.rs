use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use bincode::{serialize_into, deserialize_from};
use std::io::{BufWriter, BufReader};

use async_trait::async_trait;

pub type SentenceId = u32;
pub type WordNr = u32;

// consider Option instead of an artificial 'null'
pub const EMPTY_WORD: u32 = std::u32::MAX;

pub fn build_directory_string(mut dir: String, bin_file: &str) -> String {
    if dir.chars().last().expect("Directory string empty.") != '/' {
        dir.push_str("/");
    }
    dir.push_str(bin_file);
    dir
}

pub fn serialize_with_directory<T: Serialize>(selfs: &T, dir: String, 
                                              bin_file: &str) {
    serialize(selfs, &build_directory_string(dir, bin_file)); 
}

pub fn serialize<T: Serialize>(selfs: &T, bin_file: &str) {

    println!("start writing binary file {}.", bin_file);

    let mut f = BufWriter::new(
        File::create(bin_file)
        .expect("could not create file to persist binary data."));

    serialize_into(&mut f, selfs).unwrap();

    println!("done writing binary file.");

}

pub fn deserialize_with_directory<T: DeserializeOwned>(
    dir: String, bin_file: &str) -> T {
    deserialize(& build_directory_string(dir, bin_file)) 
}

pub fn deserialize<T: DeserializeOwned>(bin_file: &str) -> T {

    println!("start reading binary file {}.", bin_file);

    let mut f = BufReader::new(
        File::open(bin_file).unwrap());

    let o = deserialize_from(&mut f).unwrap();

    println!("done reading binary file.");

    o 
}

#[derive(Debug)]
pub struct WPair {
    pub w1: WordNr,
    pub w2: WordNr,
    pub fitness: i16
}

impl Clone for WPair {
   fn clone(&self) -> WPair {
        WPair {
            w1: self.w1,
            w2: self.w2,
            fitness: self.fitness
        }
    }
}

impl WPair {
    pub fn new(w1: WordNr, w2: WordNr) -> WPair {
        WPair {
           w1, w2,
           fitness: 0i16 
        }
    }

    pub fn new_str(w1: &str, w2: &str, env: &Env) -> WPair {

        let w1 = env.dict.get_opt_nr(w1).expect("w1 not found in dict.");
        let w2 = env.dict.get_opt_nr(w2).expect("w2 not found in dict.");

        WPair::new(w1, w2)
    }

    pub fn println(&self, env: &Env) {
        println!("fitness: {}, w1: {}, w2: {}",
                 self.fitness,
                 if self.w1 == EMPTY_WORD { "empty" }
                 else { &env.dict.get_word(&self.w1) },
                 if self.w2 == EMPTY_WORD { "empty" }
                 else { &env.dict.get_word(&self.w2) });
    }
}

pub struct Pattern {
    pub prefix: WordNr,
    pub infix: Vec<WordNr>,
    pub suffix: WordNr,
    pub order: bool,
    pub fitness: i16
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
    pub fn new(prefix: WordNr, infix: Vec<WordNr>,
               suffix: WordNr, order: bool) -> Pattern {
        Pattern {
            prefix, infix, suffix, order,
            fitness: 0i16
        }
    }

    pub fn println(&self, env: &Env) {
        println!("fitness: {}, prefix: {}, infix: {:?}, suffix: {}, order: {}",
                 self.fitness,

                 if self.prefix == EMPTY_WORD { "empty" }
                 else { env.dict.get_word(&self.prefix) },

                 {
                     self.infix.iter()
                         .map(|word_nr|
                              env.dict.get_word(&word_nr))
                         .collect::<Vec<&str>>()
                 },

                 if self.suffix == EMPTY_WORD { "empty" }
                 else { &env.dict.get_word(&self.suffix) },

                 self.order);
    }
}

#[derive(Serialize, Deserialize)]
pub struct InvertedIndex {
    pub inverted_idx: HashMap<WordNr, HashSet<SentenceId>>,
}

impl InvertedIndex {
    const FILE_NAME: &'static str = "inv_idx.bin";

    pub fn new() -> InvertedIndex {
        InvertedIndex {
            inverted_idx: HashMap::new()
        }
    }

    pub fn serialize(&self, dir: String) {
        serialize_with_directory(self, dir, &InvertedIndex::FILE_NAME);
    }

    pub fn deserialize(dir: String) -> InvertedIndex {
        deserialize_with_directory(dir, &InvertedIndex::FILE_NAME)
    }

}

#[derive(Serialize, Deserialize)]
pub struct Sentences {
    pub sentences: Vec<Vec<WordNr>>
}

impl Sentences {
    pub const FILE_NAME: &'static str = "sent.bin";

    pub fn new() -> Sentences {
        Sentences {
            // TODO think about linked list?
            sentences: Vec::new()
        }
    }

    pub fn serialize(&self, dir: String) {
        serialize_with_directory(self, dir, Sentences::FILE_NAME);
    }

    pub fn deserialize(dir: String) -> Sentences{
        deserialize_with_directory(dir, &Sentences::FILE_NAME)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dict {
    pub dict_vec: Vec<String>,
    pub dict: HashMap<String, WordNr>,
}

impl Dict {
    pub const FILE_NAME: &'static str = "dict.bin";

    pub fn new() -> Dict {
        Dict {
            dict_vec: Vec::new(),
            dict: HashMap::new()
        }
    }

    // do not return a reference (8 byte) on a
    // 4 byte number, but copy the number instead
    pub fn get_nr (&self, w: &str) -> WordNr {
        self.dict[w]
    }

    pub fn get_opt_nr (&self, w: &str) -> Option<WordNr> {
        self.dict.get(w).copied()
    }

    pub fn get_word <'a> (&'a self, n: &WordNr ) -> &'a str{
        & self.dict_vec[*n as usize]
    }
    
    pub fn serialize(&self, dir: String) {
        serialize_with_directory(self, dir, Dict::FILE_NAME);
    }

    pub fn deserialize(dir: String) -> Dict {
        deserialize_with_directory(dir, &Dict::FILE_NAME)
    }
}

pub struct Env {
    pub sentences: Sentences,
    pub inverted_idx: InvertedIndex,
    pub dict: Dict,
    pub _pairs: Vec<WPair>,
    pub the: WordNr
}

impl Env {
    pub fn new() -> Env {
        Env {
            sentences: Sentences::new(),
            inverted_idx: InvertedIndex::new(),
            dict: Dict::new(),
            _pairs: Vec::new(), 
            the: EMPTY_WORD
        }
    }

    pub fn get_inverted_idx(&self, w: &WordNr) -> &HashSet<SentenceId> {
        self.inverted_idx.inverted_idx.get(w)
            .unwrap_or_else(
                || panic!("No inverted index entry for word number {}.", w))
    }

    pub fn get_sentence(&self, s_id: &SentenceId) -> &Vec<WordNr> {
        &self.sentences.sentences[*s_id as usize]
    }

    pub fn add_word(&mut self, w: &str) -> WordNr {
        if self.dict.dict.contains_key(w) {
            self.dict.dict[w]
        } else {
            let i = self.dict.dict_vec.len() as WordNr; 

            //TODO rly two copies needed?
            self.dict.dict_vec.push(w.to_owned());
            self.dict.dict.insert(w.to_owned(), i);
            i
        }
    }

    pub fn add_inv_idx(&mut self, w: WordNr, s_id: SentenceId) {
        self.inverted_idx.inverted_idx.entry(w)
            .or_insert_with(HashSet::new)
            .insert(s_id);
    }

    pub fn serialize(&self, dir: String) {
        self.inverted_idx.serialize(dir.clone());
        self.sentences.serialize(dir.clone());
        self.dict.serialize(dir);
    }

    pub fn deserialize(dir: String) -> Env {
        let mut e = Env::new();
        e.dict = Dict::deserialize(dir.clone());
        e.sentences = Sentences::deserialize(dir.clone());
        e.inverted_idx = InvertedIndex::deserialize(dir);
        e
    }
}

#[derive(Serialize, Deserialize)]
pub struct DipreInput {
    pub pairs: Vec<(String, String)>
}

impl DipreInput {
    pub fn new(pairs: Vec<(&str, &str)>) -> DipreInput {
        DipreInput {
            pairs: pairs.iter()
                .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
                .collect()
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self)
            .expect("Could not serialize DipreInput to JSON String")
    }

    pub fn deserialize(s: &str) -> DipreInput {
        serde_json::from_str(s)
            .expect("Could not deserialize JSON String to DipreInput")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoocInput {
    pub set: Vec<String>
}

impl CoocInput {
    pub fn new(set: Vec<&str>) -> CoocInput {
        CoocInput {
            set: set.iter()
                .map(|a| (*a).to_string())
                .collect()
        }
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self)
            .expect("Could not serialize CoocInput to JSON String")
    }

    pub fn deserialize(s: &str) -> CoocInput {
        serde_json::from_str(s)
            .expect("Could not deserialize JSON String to CoocInput")
    }
}
    
#[async_trait]
pub trait AsyncLogger {
    async fn log(&mut self, s: String) -> ();
}

pub struct CoocFst {
    word: WordNr,
    fitness: i16
}

pub struct CoocSnd {
    word: WordNr,
    fitness: i16
}
