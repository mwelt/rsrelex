use super::pso::{Position, Bound, Swarm, FitnessFn};
use super::conex::{do_conex, ConexHyperParameter};
use super::types::{CoocInput, WordNr, Env};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use log::{debug, info};

pub fn read_word_file(file_name: &str, env: &Env) -> Vec<WordNr> {
    let f = File::open(file_name)
        .unwrap_or_else(|_| panic!("Unable to open word file \"{}\".", file_name));

    let lines: Vec<String> = BufReader::new(f).lines()
        .collect::<Result<Vec<String>>>()
        .unwrap_or_else(|_| panic!("Unable to read word file \"{}\".", file_name));

    let mut count_missing: usize = 0;

    let word_nrs: Vec<WordNr> = lines.iter()
        .filter_map(|s| {
            let o_wnr = env.dict.get_opt_nr(s);
            if o_wnr.is_none() {
                debug!("Reference word \"{}\" not found in dictionary", s);
                count_missing += 1;
            }
            o_wnr
        }).collect();
   
    let x: usize = word_nrs.len() + count_missing;
    info!("{} from {} known words found in word file \"{}\".", 
        word_nrs.len(), x, file_name);

    word_nrs
}

pub fn calc_precision_recall(
    retrival_erg: &[WordNr],
    reference: &[WordNr]) -> (f64, f64) {

    let retrival_erg: HashSet<&WordNr> = retrival_erg.iter().collect();
    let reference: HashSet<&WordNr> = reference.iter().collect();

    let true_positives: HashSet<_> = retrival_erg.intersection(&reference).collect();
    // let false_positives = retrival_erg.difference(&true_positives)
    //     .map(|x| *x).collect();
    // let false_negatives = reference.difference(&true_positives)
    //     .map(|x| *x).collect();

    let precision = true_positives.len() as f64 / retrival_erg.len() as f64;
    let recall = true_positives.len() as f64 / reference.len() as f64;

    (precision, recall)
}

pub fn train_mopso<'a>(
    bootstrap_words: &CoocInput, 
    reference_words: &Vec<WordNr>, 
    dat_dir: &str, 
    env: &Env) {

    let fitness_fn = |p: &Position| {
        let hyper_params = ConexHyperParameter::from_vector(p.to_vec());
        let conex_res = do_conex(bootstrap_words, &hyper_params, env);
        let (precision, recall) = calc_precision_recall(&conex_res, reference_words);
        vec![precision, recall]
    };

    // {
    // };

    let position_bounds: Vec<Bound> = vec![
        (std::f64::MIN, std::f64::MAX),
        (std::f64::MIN, std::f64::MAX),
        (std::f64::MIN, std::f64::MAX),
        (std::f64::MIN, std::f64::MAX),
        (std::f64::MIN, std::f64::MAX),
        (std::f64::MIN, std::f64::MAX),
        (std::f64::MIN, std::f64::MAX)
        ];

    let fitness_bounds: Vec<Bound> = vec![ 
        (1.0, 0.0),
        (1.0, 0.00)
        ];

    let fitness_pareto_directions = vec![true, true];
    
    let mut swarm = Swarm::new(
        50,
        0.1,
        0.1,
        0.02,
        position_bounds,
        fitness_bounds,
        vec![true, false],
        &fitness_fn
    );
}
