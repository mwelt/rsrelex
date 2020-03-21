use super::types::{SentenceId, Env, sanity_test};

use std::fs::{read_dir};

use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

pub type PreprocessorFunction = fn(&str) -> String;

pub fn read_xml_and_persist_env( 
    input_dir: &str, 
    output_dir: &str, 
    tag: &[u8], 
    limit: Option<usize>, 
    preprocessor: Option<&PreprocessorFunction>) {

    println!("starting read_xml_and_persist_env.");
    println!("reading files from directory {}.", input_dir);

    let files = file_names_from_directory(input_dir)
        .expect("Unable to read file names from directory {}!");

    let env = read_xmls_to_env(&files, tag, limit, preprocessor);

    println!("done reading files from directory.");
    
    println!("{} sentences loaded, with {} distinct words."
             , env.sentences.sentences.len(), env.dict.dict_vec.len()); 

    println!("Starting sanity test.");
    sanity_test(&env);
    println!("Done sanity test.");

    println!("starting writing binary files.");
    env.serialize(output_dir.to_owned());
    println!("done writing binary files.");

    println!("done read_and_serialize_xmls.");

}

fn file_names_from_directory(dir: &str) -> std::io::Result<Vec<String>> {
    let mut r = Vec::new();
    for elem in read_dir(dir)? {
        let p = elem?.path();
        if ! p.is_dir() {
            r.push(p.to_str().unwrap().to_owned());
        }
    }
    Ok(r)
}

fn read_xmls_to_env (
    files: &[String], 
    tag: &[u8], 
    limit: Option<usize>, 
    preprocessor: Option<&PreprocessorFunction>) -> Env {

    let mut env = Env::new();
    let mut count = 0usize;

    for file_name in files {
        count += process_xml_file(&file_name, tag,
            &mut env, limit.map(|l| l - count), preprocessor);

        if limit.is_some() && count >= limit.unwrap() { break; }
    }

    env
}

fn process_xml_file(
    file_name: &str, 
    tag: &[u8], 
    env: &mut Env, 
    limit: Option<usize>,
    preprocessor: Option<&PreprocessorFunction>) -> usize {

    let mut reader = Reader::from_file(file_name)
        .expect("Could not read from input file.");

    let mut buf = Vec::new();

    let mut read: bool = false;

    let mut curr_str = String::new();
    let mut count = 0usize;

    println!("Start reading file {}", file_name);

    loop {
        match reader.read_event(&mut buf) {

            Ok(Event::Start(ref e)) => {
                if tag == e.name() {
                    count+=1;
                    if count % 100 == 0 {
                        println!("count: {}", count);
                    }
                    if limit.is_some() && count >= limit.unwrap() {
                        break;
                    }
                    read = true;
                }
            },

            Ok(Event::End(ref e)) => {
                if tag == e.name() {

                    // optional preprocessor
                    curr_str = if let Some(p_fn) = preprocessor {
                        p_fn(&curr_str)
                    } else { curr_str };

                    let mut sentences = curr_str.unicode_sentences()
                        .map(|sent| sent
                             .split_word_bounds()
                             .filter(|word| *word != " ")
                             .map(|word| env.add_word(word))
                             .collect::<Vec<u32>>())
                        .collect::<Vec<Vec<u32>>>();

                    for (i, sent) in sentences.iter().enumerate() {
                        let sentence_id: SentenceId =
                            (i + env.sentences.sentences.len()) as u32; 
                        for word in sent {
                            env.add_inv_idx(*word, sentence_id);
                        }
                    }

                    env.sentences.sentences.append(&mut sentences);

                    curr_str = String::new(); 
                    read = false;
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
    count
}
