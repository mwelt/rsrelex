pub mod service;
pub mod types;
pub mod relex;
pub mod conex;
pub mod wikitext;
pub mod xml;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests;

use types::{DefaultLogger, sanity_test, Env, DipreInput};
use xml::read_xml_and_persist_env;
use std::env;
use relex::do_relex;
// use std::collections::HashMap;
use getopts::Options;

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

fn print_usage(program: &str, opts: Options){
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

#[tokio::main]
async fn main() {
// fn main() {

    let args: Vec<String> = env::args().collect(); 
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("s", "sanity", "test sanity of bin-files");
    opts.optopt("x", "import-xml", "import xml files from directory", "DIR");
    opts.optopt("t", "tag", "read specific tag from xml files.", "TAG");
    opts.optopt("l", "limit", "Limit the count of documents processed from all xml files.", "LIMIT");
    opts.optopt("p", "preprocessor", "Preprocessor function.", "FUNC");
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

        if ! matches.opt_present("t") {
            eprintln!("If x option present t needs to be present as well!");
            print_usage(&program, opts);
            return;
        }

        let tag = match matches.opt_str("t") {
            None => {
                eprintln!("If x option present t needs to be present as well!");
                print_usage(&program, opts);
                return;
            }
            Some(t) => { t }
        };

        // read_xml_and_persist_env(&input_dir, &bin_file_dir, b"AbstractText", Option::Some(1000));
        let limit: Option<usize> = if matches.opt_present("l") {
            matches.opt_str("l").and_then(|l| l.parse().ok()) 
        } else { Option::None };

        // let preprocessors: HashMap<String, &dyn Fn(&str) -> String> = 
        //     HashMap::new();

        // preprocessors.insert("wikitext::strip_markup".into(), 
        //     &wikitext::strip_markup);

        let preprocessor: Option<&dyn Fn(&str) -> String> = if matches.opt_present("p") {
            matches.opt_str("p")
                .and_then(|p| {
                    if p == "wikitext::strip_markup" {
                        println!("using wikitext::strip_markup as preprocessor");
                        Option::Some(&wikitext::strip_markup as &dyn Fn(&str) -> String)
                    } else {
                        Option::None
                    }
                })
        } else {
            Option::None
        };

       read_xml_and_persist_env(
           &input_dir, 
           &bin_file_dir, 
           &tag.as_bytes(), 
           limit, 
           preprocessor);

    } else {
        let env = bootstrap(bin_file_dir);

        if matches.opt_present("s") {
            println!("Starting sanity test.");
            sanity_test(&env);
            println!("Done sanity test.");
        }

        let wpairs = vec! [
            ("organs", "liver"),
            ("organs", "lung"),
            ("animal", "cat"),
            ("animal", "dog")
        ];
        let json = DipreInput::new(wpairs).serialize();
        println!("Json: {}", json);
        do_relex(DipreInput::deserialize(&json), &env, DefaultLogger::new()).await;

        // service::run_server(env).await;    
    }
        
        // start server
        // let cooc_input = CoocInput::new(vec![ "SLC1A5",
        //                                 "CXADR",
        //                                 "CAV2",
        //                                 "NUP98",
        //                                 "CTBP2",
        //                                 "GSN",
        //                                 "HSPA1B",
        //                                 "STOM",
        //                                 "RAB1B",
        //                                 "HACD3",
        //                                 "ITGB6",
        //                                 "IST1",
        //                                 "NUCKS1",
        //                                 "TRIM27",
        //                                 "APOE",
        //                                 "SMARCB1",
        //                                 "UBP1",
        //                                 "CHMP1A",
        //                                 "NUP160",
        //                                 "HSPA8",
        //                                 "DAG1",
        //                                 "STAU1",
        //                                 "ICAM1",
        //                                 "CHMP5",
        //                                 "DEK",
        //                                 "VPS37B",
        //                                 "EGFR",
        //                                 "CCNK",
        //                                 "PPIA",
        //                                 "IFITM3",
        //                                 "PPIB",
        //                                 "TMPRSS2",
        //                                 "UBC",
        //                                 "LAMP1",
        //                                 "CHMP3"]);
        // conex::do_conex(cooc_input, &env);
    // }

    // // let wpairs = vec![
    // //     ("organs", "liver"),
    // //     ("organs", "lung"),
    // //     ("bacteria", "Staphylococcus"),
    // //     ("bacteria", "Streptococcus"),
    // //     ("organs", "esophagus")
    // //     // ("cancer", "BRCA1"),
    // //     // ("cancer", "UV"),
    // //     // ("cancer", "ultraviolet"),
    // //     // ("cancer", "alcohol"),
    // //     // ("cancer", "tobacco"),
    // // ];



}
