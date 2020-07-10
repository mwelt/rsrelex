pub mod service;
pub mod types;
pub mod relex;
pub mod conex;
pub mod wikitext;
pub mod xml;
pub mod pso;
pub mod mopso;
pub mod pso_train;
pub mod mopso_train;
pub mod utils;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wikitext_tests;
#[cfg(test)]
mod mopso_tests;
#[cfg(test)]
mod pso_tests;
#[cfg(test)]
mod pso_train_tests;
#[cfg(test)]
mod utils_tests;

use serde::{Serialize, Deserialize};
use toml;
use log::{info, error};
use types::{WordNr, soundness_test, Env};
use xml::{read_xml_and_persist_env, PreprocessorFunction};
use std::env;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::{self, BufRead};
use getopts::{Matches, Options};
use rand::seq::SliceRandom;
use atty::Stream;

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

fn print_usage(program: &str, opts: &Options){
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn run_xml_import(
    opts: &Options, 
    matches: &Matches, 
    program: &str, 
    bin_file_dir: String){
    
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
}

fn run_training(
    opts: &Options, 
    matches: &Matches, 
    program: &str, 
    env: &Env
    ){

    let reference_file = match matches.opt_str("t") {
        None => {
            print_usage(&program, opts);
            return;
        }
        Some(t) => { t }
    };

    let outfile = match matches.opt_str("to") {
        None => {
            "pso_train_swarm.dat".to_string()
        }
        Some(t) => { t }
    };

    let do_mopso = matches.opt_present("tmopso");            

    let num_particles: usize = if matches.opt_present("tnparticles") {
        matches.opt_str("tnparticles")
            .and_then(|l| l.parse().ok()).unwrap_or(100)
    } else { 100 };

    let iterations: usize = if matches.opt_present("tniter") {
        matches.opt_str("tniter")
            .and_then(|l| l.parse().ok()).unwrap_or(100)
    } else { 100 };

    let nbwords: usize = if matches.opt_present("tnbwords") {
        matches.opt_str("tnbwords")
            .and_then(|l| l.parse().ok()).unwrap_or(5)
    } else { 5 };

    let reference_words = utils::read_word_file(&reference_file, &env); 

    // check if run from a pipe
    let bootstrap_words: HashSet<WordNr> = 
        // if no pipe, randomize tnbwords from the referenece_words
        if atty::is(Stream::Stdin) {

            let mut rng = rand::thread_rng();

            let bootstrap_words: HashSet<WordNr> = reference_words
                .choose_multiple(&mut rng, nbwords)
                .cloned().collect();

            info!("Using {} random bootstrap_words: {:?}", nbwords,
                bootstrap_words.iter().map(|w_nr| env.dict.get_word(w_nr))
                .collect::<Vec<&str>>());

            bootstrap_words
        }
        // else read a wordlist from stdin
        else{

            let bootstrap_words: HashSet<WordNr> = utils::read_words_from_stdin(&env)
                .iter().cloned().collect();

            info!("Using bootstrap words from stdin. --tnbwords option is ignored!"); 

            bootstrap_words
        };

    if std::path::Path::new(&outfile.clone()).exists() {
        info!("{} already exists, removing.", outfile);
        std::fs::remove_file(&outfile)
            .unwrap_or_else(|_| panic!("unable to delete {}", outfile));
    }

    if !do_mopso {
        let fitness_fn = pso_train::ConexFitnessFn::new(
            &bootstrap_words,
            &reference_words,
            env 
        );
        info!("starting pso training.");
        let winner_hyper_params = 
            pso_train::train(num_particles, iterations, &fitness_fn, &outfile);
        info!("finished pso training.");

        info!("Winner Configuration: {:?}", winner_hyper_params);
        let final_run_result = conex::do_conex(&types::CoocInput { 
            set: bootstrap_words.iter()
                .map(|w_nr| env.dict.get_word(w_nr).to_owned())
                .collect()
        }, &winner_hyper_params, env);

        info!("Winner Result: {:?}", final_run_result.iter()
            .map(|w_nr| env.dict.get_word(w_nr)).collect::<Vec<&str>>());

    } else {
        let fitness_fn = mopso_train::ConexFitnessFn::new(
            &bootstrap_words,
            &reference_words,
            env 
        );
        info!("starting mopso training.");
        mopso_train::train(num_particles, iterations, &fitness_fn, &outfile);
        info!("finished mopso training.");
    }

    // info!("final leader:");
    // info!("Position: {:?}, Fitness: {:?}", p, f);
}

fn run_server(){

    // service::run_server(env).await;    

    // let set = vec! [
    //     "London",
    //     "Berlin",
    //     "Madrid",
    //     "Lima"
    // ];

    // info!("{:?}", set);
    // let json = CoocInput::new(set);
    // do_conex(&json, &conex::DEFAULT_CONEX_HYPER_PARAMETER, &env);
}

fn run_relex(
    opts: &Options, 
    matches: &Matches, 
    program: &str, 
    env: &Env
    ){

    // let config_file = match matches.opt_str("c") {
    //     None => {
    //         print_usage(&program, opts);
    //         return;
    //     }
    //     Some(t) => { t }
    // };

   
    // let config: ConexConfig = 
    //     toml::from_str(&read_to_string(&config_file)
    //     .unwrap_or_else(|_| panic!("Unable to open file \"{}\".", &config_file)))
    //     .unwrap_or_else(|_| panic!("Unable to open file \"{}\".", &config_file));

    // conex::do_conex(&types::CoocInput{ set: config.seed_terms }, 
    //     &config.hyper_parameter, 
    //     env);
}

#[derive(Serialize, Deserialize, Default)]
struct ConexConfig {
    hyper_parameter: conex::ConexHyperParameter,
    seed_terms: Vec<String>
}

fn run_conex(
    opts: &Options, 
    matches: &Matches, 
    program: &str, 
    env: &Env
    ){

    let config_file = match matches.opt_str("c") {
        None => {
            print_usage(&program, opts);
            return;
        }
        Some(t) => { t }
    };

   
    let config: ConexConfig = 
        toml::from_str(&read_to_string(&config_file)
        .unwrap_or_else(|_| panic!("Unable to open file \"{}\".", &config_file)))
        .unwrap_or_else(|_| panic!("Unable to read file \"{}\".", &config_file));

    let result_words = conex::do_conex(&types::CoocInput{ set: config.seed_terms }, 
        &config.hyper_parameter, 
        env);

    println!("{:?}", result_words.iter()
        .map(|w_nr| env.dict.get_word(w_nr)).collect::<Vec<&str>>());
}

// #[tokio::main]
// async fn main() {
fn main() {

    env_logger::init();

    let args: Vec<String> = env::args().collect(); 
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("s", "soundness", "Test soundness of bin-files.");
    opts.optopt("d", "deamon", "Starts REST Server backend.", "PORT");
    opts.optopt("r", "relex", "Starts RELEX with specified input.", "FILE");
    opts.optopt("c", "conex", "Starts CONEX with specified input.", "FILE");
    opts.optopt("t", "train", 
        "Train model parameter with PSO / MOPSO (--tmopso).", "FILE");
    opts.optopt("", "to", "Training outputfile.", "FILE");
    opts.optflag("", "tmopso", "Training with mopso.");
    opts.optopt("", "tnparticles", "Num particles. (defaults to 100)", "NUM");
    opts.optopt("", "tniter", "Num iterations. (defaults to 100)", "NUM");
    opts.optopt("", "tnbwords", "Num bootstrap words. (defaults to 5)", "NUM");
    opts.optopt("x", "import-xml", "Import xml files from directory.", "DIR");
    opts.optopt("", "xt", "Read specific tag from xml files.", "TAG");
    opts.optopt("", "xl", 
        "Limit the count of documents processed from all xml files.", "LIMIT");
    opts.optopt("", "xp", "Preprocessor function.", "FUNC");
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
            print_usage(&program, &opts);
            return;
        }
        Some(d) => { d }
    };

    if matches.opt_present("x") {
        run_xml_import(&opts, &matches, &program, bin_file_dir);
    } else {
        let env = bootstrap(bin_file_dir);

        if matches.opt_present("s") {
            info!("Starting soundness test.");
            soundness_test(&env);
            info!("Done soundness test.");
    
        } else if matches.opt_present("r") {
            run_relex(&opts, &matches, &program, &env);
        } else if matches.opt_present("c") {
            run_conex(&opts, &matches, &program, &env);
        } else if matches.opt_present("t") {
            run_training(&opts, &matches, &program, &env);
        } else if matches.opt_present("") {
            run_server();
        }
    }

}
