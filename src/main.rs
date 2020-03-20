pub mod service;
pub mod types;
pub mod dipre;
pub mod cooc;
pub mod wikitext;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests;

use types::{SentenceId, Env, CoocInput, WordNr};

use std::env;
use std::fs;

use unicode_segmentation::UnicodeSegmentation;
use quick_xml::Reader;
use quick_xml::events::Event;

use getopts::Options;

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

fn read_and_serialize_xmls(input_dir: &str, output_dir: &str){
    println!("starting read_and_serialize_xmls.");
    let mut env = Env::new();

    // let input_dir: &str = args.get(1).expect("no input directory provided.");
    // let output_dir: &str = args.get(2).expect("no output directory provided.");

    println!("reading files from directory {}.", input_dir);
    for file_name in file_names_from_directory(input_dir)
        .expect("could not read input directory.") {

            read_xml_file(&file_name, &mut env);
        }

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

fn bootstrap(dir: String) -> Env {
    println!("bootstraping.");

    println!("start reading binary data.");
    let mut env = Env::deserialize(dir);
    println!("done reading binary data.");

    println!("{} sentences loaded, with {} distinct words."
             , env.sentences.sentences.len(),
             env.dict.dict_vec.len()); 

    env.the = env.dict.get_opt_nr("the")
        .expect("\"the\" not found in dictionary.");

    env
}

fn sanity_test(env: &Env){
    // check if every dictionary word is associated with 
    // an inverted index entry

    let mut lost_words: Vec<WordNr> = Vec::new();

    for w_nr in env.dict.dict.values() {
        if ! env.inverted_idx.inverted_idx.contains_key(w_nr) {
            lost_words.push(*w_nr);  
        }
    }

    if ! lost_words.is_empty() {
        println!("Words in dictionary without inverted_index entry:\n{:?}",
               lost_words.iter().map(
                   |w_nr| env.dict.get_word(w_nr)).collect::<Vec<&str>>());
        panic!("Sanity check failed!");
    }
}

fn print_usage(program: &str, opts: Options){
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

// #[tokio::main]
// async fn main() {
fn main() {

    let args: Vec<String> = env::args().collect(); 
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("s", "sanity", "test sanity of bin-files");
    opts.optopt("x", "import-xml", "import xml files from directory", "DIR");
    opts.reqopt("b", "bin-files", "bin-file directory (if -x is present this directory denotes the
        output directory, otherwise bin-file backup data is read from this directory).", "DIR");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    let bin_file_dir = match matches.opt_str("b") {
        None => { 
            print_usage(&program, opts);
            return;
        }
        Some(d) => { d }
    };


    if matches.opt_present("x") {
        // do xml import
        let input_dir = match matches.opt_str("x") {
            None => { 
                print_usage(&program, opts);
                return;
            }
            Some(d) => { d }
        };
        read_and_serialize_xmls(&input_dir, &bin_file_dir);
    } else {
        let env = bootstrap(bin_file_dir);

        if matches.opt_present("s") {
            println!("Starting sanity test.");
            sanity_test(&env);
            println!("Done sanity test.");
        }
        
        // start server
        let cooc_input = CoocInput::new(vec![ "SLC1A5",
                                        "CXADR",
                                        "CAV2",
                                        "NUP98",
                                        "CTBP2",
                                        "GSN",
                                        "HSPA1B",
                                        "STOM",
                                        "RAB1B",
                                        "HACD3",
                                        "ITGB6",
                                        "IST1",
                                        "NUCKS1",
                                        "TRIM27",
                                        "APOE",
                                        "SMARCB1",
                                        "UBP1",
                                        "CHMP1A",
                                        "NUP160",
                                        "HSPA8",
                                        "DAG1",
                                        "STAU1",
                                        "ICAM1",
                                        "CHMP5",
                                        "DEK",
                                        "VPS37B",
                                        "EGFR",
                                        "CCNK",
                                        "PPIA",
                                        "IFITM3",
                                        "PPIB",
                                        "TMPRSS2",
                                        "UBC",
                                        "LAMP1",
                                        "CHMP3"]);
        cooc::do_cooc(cooc_input, &env);
    }

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

    // service::run_server(env).await;    


}
