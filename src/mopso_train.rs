use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use std::io::Write;
use std::fs::{File, OpenOptions};

fn append_swarm_to_file(s: &mopso::Swarm, f: &str){

   let mut file = OpenOptions::new()
       .create(true)
       .append(true)
       .open(f)
       .expect(&format!("Unable to open {}.", f));

    s.particles.axis_iter(Axis(0)).for_each(|p|
        writeln!(file, "{}", 
            p.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\t")).unwrap());

    //write new line to seperate leaders
    writeln!(file, "\n").unwrap();

    s.leaders.axis_iter(Axis(0)).for_each(|l| 
        writeln!(file, "{}",
            l.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\t")).unwrap());

    //write new line to seperate iterations 
    writeln!(file, "\n").unwrap();

}

pub struct ConexFitnessFn<'a> {
    env: &'a Env, 
    bootstrap_words: &'a HashSet<WordNr>, 
    reference_words: &'a Vec<WordNr>
}

#[allow(clippy::ptr_arg)]
impl ConexFitnessFn<'_> {
    pub fn new<'a>(
        bootstrap_words: &'a HashSet<WordNr>, 
        reference_words: &'a Vec<WordNr>,
        env: &'a Env) -> ConexFitnessFn<'a> {

        ConexFitnessFn {
            env,
            bootstrap_words,
            reference_words
        }
    }

    pub fn fitness(&self, swarm: &mut mopso::Swarm){
        let o = &swarm.o;
        swarm.particles.axis_iter_mut(Axis(0))
            .into_par_iter()
            .for_each(|mut p| {
                let pos = p.slice(s![o.p.0..o.p.1]); 
                let hyper_params = 
                    conex::ConexHyperParameter::from_vector(pos.to_vec(), 0f64);
                let conex_res = 
                    conex::do_conex_(self.bootstrap_words, &hyper_params, self.env);
                let (precision, recall) = 
                    pso_train::calc_precision_recall(&conex_res, self.reference_words);

                p[o.f.0] = precision;
                p[o.f.0+1] = recall;

            });
    }
}

pub fn train<'a>(
    num_particles: usize,
    iterations: usize,
    fitness: &'a ConexFitnessFn,
    out_file: &'a str){

    let position_bounds = array![
        [ -100f64, 100f64 ], // cooc1_word_frequency_boost
        [ -100f64, 100f64 ], // cooc1_set_frequency_boost
        [ -1000f64, 1000f64 ], // cooc1_global_term_frequency_boost_per_sentence
        [ -100f64, 100f64 ], // cooc2_word_frequency_boost
        [ -100f64, 100f64 ], // cooc2_word_frequncy_boost
        [ -1000f64, 1000f64 ], // cooc2_global_term_frequency_boost_per_sentence
        ];
  
    let mut swarm = mopso::Swarm::new(
       num_particles, 
       6,
       position_bounds,
       2,
       array![
       [0.0, 1.0],
       [0.0, 1.0],
       ],
       array![true, true],
       0,
       &|swarm: &mut mopso::Swarm| {
           fitness.fitness(swarm);
       },
       &|_i: usize, swarm: &mut mopso::Swarm| {
           info!("{} of {}", _i, iterations);
           append_swarm_to_file(swarm, out_file);
       }
    );

    swarm.fly(
        iterations,
        &pso::HyperParams {
            learning_cognitive: 0.2,
            learning_social: 0.2,
            inertia: 0.02
        },
        &|swarm: &mut mopso::Swarm| {
            fitness.fitness(swarm);
        },
        &|_i: usize, swarm: &mut mopso::Swarm| {
            info!("{} of {}", _i, iterations);
            append_swarm_to_file(swarm, out_file);
        });
//     let mut swarm: Swarm<'a, ConexFitnessFn> = Swarm::<ConexFitnessFn>::new(
//         500,
//         0.2,
//         0.2,
//         0.02,
//         position_bounds,
//         (-1.0, 2.0),
//         fitness_fn,
//         &|i, payloads, swarm| {
//             write_swarm_data(i, swarm, dat_dir);
//         }
//     );

//     swarm.fly(100,
//         &|i, payloads, swarm| {
//             write_swarm_data(i, swarm, dat_dir);
//         });
}

// use super::mopso::{FitnessFn, Position, Fitness, Swarm, Bound};
// use super::conex::{do_conex_, ConexHyperParameter};
// use super::types::{WordNr, Env};
// use std::collections::HashSet;
// use std::fs::{File, write};
// use std::io::{BufRead, BufReader, Result};
// use log::{debug, info};

// pub fn read_word_file(file_name: &str, env: &Env) -> Vec<WordNr> {
//     let f = File::open(file_name)
//         .unwrap_or_else(|_| panic!("Unable to open word file \"{}\".", file_name));

//     let lines: Vec<String> = BufReader::new(f).lines()
//         .collect::<Result<Vec<String>>>()
//         .unwrap_or_else(|_| panic!("Unable to read word file \"{}\".", file_name));

//     let mut count_missing: usize = 0;

//     let word_nrs: Vec<WordNr> = lines.iter()
//         .filter_map(|s| {
//             let o_wnr = env.dict.get_opt_nr(s);
//             if o_wnr.is_none() {
//                 debug!("Reference word \"{}\" not found in dictionary", s);
//                 count_missing += 1;
//             }
//             o_wnr
//         }).collect();
   
//     let x: usize = word_nrs.len() + count_missing;
//     info!("{} from {} known words found in word file \"{}\".", 
//         word_nrs.len(), x, file_name);

//     word_nrs
// }

// pub fn calc_precision_recall(
//     retrival_erg: &[WordNr],
//     reference: &[WordNr]) -> (f64, f64) {

//     let retrival_erg: HashSet<&WordNr> = retrival_erg.iter().collect();
//     let reference: HashSet<&WordNr> = reference.iter().collect();

//     let true_positives: HashSet<_> = retrival_erg.intersection(&reference).collect();
//     // let false_positives = retrival_erg.difference(&true_positives)
//     //     .map(|x| *x).collect();
//     // let false_negatives = reference.difference(&true_positives)
//     //     .map(|x| *x).collect();

//     let retrival_erg_len = retrival_erg.len();
//     let precision = if retrival_erg_len == 0 {
//         0f64
//     } else {
//         true_positives.len() as f64 / retrival_erg_len as f64
//     };

//     let recall = if retrival_erg_len == 0 {
//         0f64
//     } else {
//         true_positives.len() as f64 / reference.len() as f64
//     };

//     (precision, recall)
// }

// pub struct ConexFitnessFn<'a> {
//     env: &'a Env, 
//     bootstrap_words: &'a HashSet<WordNr>, 
//     reference_words: &'a Vec<WordNr>
// }

// #[allow(clippy::ptr_arg)]
// impl ConexFitnessFn<'_> {
//     pub fn new<'a>(
//         bootstrap_words: &'a HashSet<WordNr>, 
//         reference_words: &'a Vec<WordNr>,
//         env: &'a Env) -> ConexFitnessFn<'a> {

//         ConexFitnessFn {
//             env,
//             bootstrap_words,
//             reference_words
//         }
//     }
// }

// impl FitnessFn for ConexFitnessFn<'_> {
//     fn calc_fitness(&self, pos: &Position) -> Fitness {
//         let hyper_params = ConexHyperParameter::from_vector(pos.to_vec(), 0f64);
//         let conex_res = do_conex_(self.bootstrap_words, &hyper_params, self.env);
//         let (precision, recall) = calc_precision_recall(&conex_res, self.reference_words);
//         vec![precision, recall]
//     }
// }

// fn points_to_string_(points: &[(&Vec<f64>, &Vec<f64>)]) -> String {
//     points.iter()
//         .map(|(xs, ys)| {
//             let xsstr: String = xs.iter()
//                 .map(|x| x.to_string()).collect::<Vec<String>>().join("\t");

//             let ysstr: String = ys.iter()
//                 .map(|y| y.to_string()).collect::<Vec<String>>().join("\t");

//             [xsstr, ysstr].join("\t")

//         })
//         .collect::<Vec<String>>().join("\n")
// }

// fn write_swarm_data<T: FitnessFn>(i: usize, swarm: &Swarm<T>, dat_dir: &str){

//     let particle_data: Vec<(&Vec<f64>, &Vec<f64>)> = swarm.particles.iter()
//         .map(|p| (&p.position, &p.fitness)).collect();
//     write([dat_dir, "p_", &i.to_string(), ".dat"].join(""), 
//         points_to_string_(&particle_data)).unwrap();
   
//     let leader_data: Vec<(&Vec<f64>, &Vec<f64>)> = swarm.leaders.iter()
//         .map(|l| (&l.position, &l.fitness)).collect();
//     write([dat_dir, "l_", &i.to_string(), ".dat"].join(""), 
//         points_to_string_(&leader_data)).unwrap();
// }

// pub fn train_mopso<'a>(
//     fitness_fn: &'a ConexFitnessFn,
//     dat_dir: &'a str) -> Vec<(Position, Fitness)> {

//     let position_bounds: Vec<Bound> = vec![
//         (-100f64, 100f64),
//         (-100f64, 100f64),
//         (-100f64, 100f64),
//         (-100f64, 100f64),
//         (-100f64, 100f64),
//         (-100f64, 100f64),
//         (-100f64, 100f64),
//         (-100f64, 100f64)
//         // (std::f64::MIN, std::f64::MAX),
//         // (std::f64::MIN, std::f64::MAX),
//         // (std::f64::MIN, std::f64::MAX),
//         // (std::f64::MIN, std::f64::MAX),
//         // (std::f64::MIN, std::f64::MAX),
//         // (std::f64::MIN, std::f64::MAX),
//         // (std::f64::MIN, std::f64::MAX)
//         ];

//     let fitness_bounds: Vec<Bound> = vec![ 
//         (0.0, 1.0),
//         (0.0, 1.0)
//         ];

//     let fitness_pareto_directions = vec![true, true];
    
//     let mut swarm: Swarm<'a, ConexFitnessFn> = Swarm::<ConexFitnessFn>::new(
//         100,
//         0.2,
//         0.2,
//         0.02,
//         position_bounds,
//         fitness_bounds,
//         fitness_pareto_directions,
//         fitness_fn
//     );

//     swarm.fly(100,
//         &|i, swarm| {
//             write_swarm_data(i, swarm, dat_dir);
//         });

//     swarm.leaders.iter()
//         .map(|l| (l.position.clone(), l.fitness.clone())).collect()
// }