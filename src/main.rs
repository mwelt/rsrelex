pub mod service;
pub mod types;
// TODO rename to relex (RELation EXtraction)
pub mod relex;
//TODO rename to conex  (CONcept EXtraction)
pub mod conex;
pub mod wikitext;
pub mod xml;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests;

use types::{sanity_test, Env, CoocInput};
use xml::read_xml_and_persist_env;
use std::env;

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
        read_xml_and_persist_env(&input_dir, &bin_file_dir, b"AbstractText", Option::None);
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
