pub mod service;
pub mod types;
pub mod dipre;

use types::{SentenceId, Env};

use std::env;
use std::fs;

use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

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
                if b"AbstractText" == e.name() {
                    read = true;
                }
            },

            Ok(Event::End(ref e)) => {
                if b"AbstractText" == e.name() {

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
}

fn read_and_serialize_xmls(args: &[String]){
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

    println!("starting writing binary files.");

    env.serialize();

    println!("done writing binary files.");

    println!("done read_and_serialize_xmls.");
}

fn bootstrap() -> Env {
    println!("bootstraping.");

    println!("start reading binary data.");
    let mut env = Env::deserialize();
    println!("done reading binary data.");

    println!("{} sentences loaded, with {} distinct words."
             , env.sentences.sentences.len(),
             env.dict.dict_vec.len()); 

    env.the = env.dict.get_opt_nr("the")
        .expect("\"the\" not found in dictionary.");

    env
}

#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect(); 

    if args.len() > 1 {
        read_and_serialize_xmls(&args);
        return;
    }

    let env = bootstrap();

    // let wpairs = vec![
    //     ("organs", "liver"),
    //     ("organs", "lung"),
    //     ("bacteria", "Staphylococcus"),
    //     ("bacteria", "Streptococcus"),
    //     ("organs", "esophagus")
    //     // ("cancer", "BRCA1"),
    //     // ("cancer", "UV"),
    //     // ("cancer", "ultraviolet"),
    //     // ("cancer", "alcohol"),
    //     // ("cancer", "tobacco"),
    // ];

    // let json = DipreInput::new(wpairs).serialize();
    // println!("Json: {}", json);
    // do_dipre(DipreInput::deserialize(&json), &env);

    service::run_server(env).await;    

}
