pub mod service;
pub mod types;
pub mod relex;
pub mod conex;
pub mod wikitext;
pub mod xml;
pub mod pso;
pub mod train;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wikitext_tests;
#[cfg(test)]
mod pso_tests;
#[cfg(test)]
mod train_tests;

use log::{info, error};
use types::{DefaultLogger, CoocInput, soundness_test, Env, DipreInput};
use xml::{read_xml_and_persist_env, PreprocessorFunction};
use std::env;
use relex::do_relex;
use conex::do_conex;
use std::collections::HashMap;
use getopts::Options;

fn bootstrap(dir: String) -> Env {
    info!("bootstraping.");

    info!("start reading binary data.");
    let mut env = Env::deserialize(dir);
    info!("done reading binary data.");

    info!("{} sentences loaded, with {} distinct words."
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

    env_logger::init();

    let args: Vec<String> = env::args().collect(); 
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("s", "soundness", "Test soundness of bin-files.");
    opts.optopt("t", "train", "Train model parameter with MOPSO.", "FILE");
    opts.optopt("x", "import-xml", "Import xml files from directory.", "DIR");
    opts.optopt("xt", "xml-tag", "Read specific tag from xml files.", "TAG");
    opts.optopt("xl", "xml-limit", "Limit the count of documents processed from all xml files.", "LIMIT");
    opts.optopt("xp", "xml-preprocessor", "Preprocessor function.", "FUNC");
    opts.reqopt("b", "bin-files", 
        "Bin-file directory (if -x is present this directory denotes the
        output directory, otherwise bin-file backup data is read from this directory).", 
        "DIR");

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

        if ! matches.opt_present("xt") {
            error!("If x option present xt (xml-tag) needs to be present as well!");
            print_usage(&program, opts);
            return;
        }

        let tag = match matches.opt_str("xt") {
            None => {
                error!("If x option present xt (xml-tag) needs to be present as well!");
                print_usage(&program, opts);
                return;
            }
            Some(t) => { t }
        };

        // read_xml_and_persist_env(&input_dir, &bin_file_dir, b"AbstractText", Option::Some(1000));
        let limit: Option<usize> = if matches.opt_present("xl") {
            matches.opt_str("xl").and_then(|l| l.parse().ok()) 
        } else { Option::None };

        let mut preprocessors: HashMap<String, PreprocessorFunction> = 
            HashMap::new();

        preprocessors.insert("wikitext::strip_markup".into(), wikitext::strip_markup);

        let preprocessor = if matches.opt_present("xp") {
            matches.opt_str("xp").and_then(|p| {
                let p_ = preprocessors.get(&p);

                if p_.is_some() {
                    info!("using preprocessor {}.", p);
                } else {
                    info!("not using preprocessor.");
                }

                p_
            })
        } else {
            info!("not using preprocessor.");
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
            info!("Starting soundness test.");
            soundness_test(&env);
            info!("Done soundness test.");
        }

        if matches.opt_present("t") {
            let reference_file = match matches.opt_str("t") {
                None => {
                    print_usage(&program, opts);
                    return;
                }
                Some(t) => { t }
            };


        } else {
            let set = vec! [
                "London",
                "Berlin",
                "Madrid",
                "Lima"
            ];

            info!("{:?}", set);
            let json = CoocInput::new(set);
            do_conex(&json, &conex::DEFAULT_CONEX_HYPER_PARAMETER, &env);
        }


        // let wpairs = vec! [
        //     ("organs", "liver"),
        //     ("organs", "lung"),
        //     ("animal", "cat"),
        //     ("animal", "dog")
        // ];

        // let json = DipreInput::new(wpairs).serialize();
        // info!("Json: {}", json);
        // do_relex(DipreInput::deserialize(&json), &env, DefaultLogger::new()).await;
       

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
