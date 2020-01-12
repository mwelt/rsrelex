use std::collections::HashMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

type SentenceId = u32;
type WordNr = u32;

fn main() {

    // let file_name = "data2/pubmed19n0587.xml"; //"data2/pubmed19n0902.xml";
    let file_name = "data2/pubmed19n0902.xml";

    let mut reader = Reader::from_file(file_name)
        .expect("Could not read from input file.");

    let mut buf = Vec::new();

    let mut dict_vec: Vec<String> = Vec::new();
    let mut dict: HashMap<String, WordNr> = HashMap::new();

    let mut sentences_orig: Vec<Vec<String>> = Vec::new();
    let mut sentences_translated: Vec<Vec<WordNr>> = Vec::new();

    let mut add_word = |w: &String| -> WordNr {
        if dict.contains_key(w) {
            return dict[w];
        } else {
            let i = dict_vec.len() as WordNr; 
            dict_vec.push(w.to_owned());
            dict.insert(w.to_owned(), i);
            return i;
        }
    };

    let mut count_sentence_id = 0u32;
    let mut inverted_idx: HashMap<WordNr, HashSet<SentenceId>> = HashMap::new();
    
    let mut add_inv_idx = |w: WordNr, s_id: SentenceId| {
        inverted_idx.entry(w)
            .or_insert(HashSet::new())
            .insert(s_id);
    };
    
    let mut read: bool = false;

    // let mut count_open: u32 = 0;
    // let mut count_end: u32 = 0;
    // let mut count_add: u32 = 0;

    println!("Starting reading file {}", file_name);

    loop {
        match reader.read_event(&mut buf) {

            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"AbstractText" => {
                        // count_open += 1;
                        read = true;
                    }, 
                    _ => (),
                }
            },

            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"AbstractText" => {
                        // count_end += 1;
                        read = false;
                    }, 
                    _ => (),
                }
            }
           
            Ok(Event::Text(ref e)) if read => {
                // count_add += 1;
                let s: String = e.unescape_and_decode(&reader)
                   .expect("Error while reading text from xml.");

                let sentences = s.unicode_sentences();
                
                let mut sentences = sentences
                    .map(|sent| sent
                         .split_word_bounds()
                         .filter(|word| *word != " ")
                         .map(|word| word.to_owned())
                         .collect::<Vec<String>>())
                    .collect::<Vec<Vec<String>>>();

                let mut sentences_translated_ = sentences.iter()
                    .map(|sent| sent.iter()
                         .map(|word| add_word(&word))
                         .collect::<Vec<u32>>())
                    .collect::<Vec<Vec<u32>>>();

                for (i, sent) in sentences_translated_.iter().enumerate() {
                    let sentence_id: SentenceId = i as u32 + count_sentence_id; 
                    for word in sent {
                        add_inv_idx(*word, sentence_id);
                    }
                }

                count_sentence_id += sentences_translated.len() as u32;

                sentences_orig.append(&mut sentences);
                sentences_translated.append(&mut sentences_translated_);
            },

            Err(e) => panic!(
                "Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buf.clear();
    }
    
    println!("done reading file.");
    // println!("encountered {} AbtractText open tags", count_open);
    // println!("encountered {} AbtractText end tags", count_end);
    // println!("added text {} times", count_add);

    println!("size of sentences vec {}", sentences_translated.len()); 
    println!("size of dict_vec {}", dict_vec.len()); 

    // println!("{:?}", sentences_orig[0]); 
    // println!("{:?}", sentences_translated[0]); 
    // println!("{:?}", sentences_translated[0].iter()
    //          .map(|word_nr| &dict_vec[*word_nr as usize])
    //          .collect::<Vec<&String>>());

    let sentences_with_and = dict.get("cancer")
        .and_then(|word_nr| inverted_idx.get(word_nr))
        .expect("No entry for word \"and\" found in dict or in inverted index.");

    println!("count sentences with word \"cancer\" {}", sentences_with_and.len());

    // println!("2nd of txt vec {:?}", txt[1]); 
    // println!("2nd of txt vec {:?}", txt[3]); 
    // println!("last of txt vec {:?}", txt[txt.len() - 1]);
}
