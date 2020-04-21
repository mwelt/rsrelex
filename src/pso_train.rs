use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use log::info;

fn append_swarm_to_file(s: &pso::Swarm, f: &str){

   let mut file = OpenOptions::new()
       .create(true)
       .append(true)
       .open(f)
       .unwrap_or_else(|_| panic!("Unable to open {}.", f));

    s.particles.axis_iter(Axis(0)).for_each(|p|
        writeln!(file, "{}", 
            p.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\t")).unwrap());

    //write new line to seperate iterations 
    writeln!(file, "\n").unwrap();
}

pub fn calc_precision_recall(
    retrival_erg: &[WordNr],
    reference: &[WordNr]) -> (f64, f64) {

    let retrival_erg: HashSet<&WordNr> = retrival_erg.iter().collect();
    let reference: HashSet<&WordNr> = reference.iter().collect();

    let true_positives: HashSet<_> = retrival_erg.intersection(&reference).collect();

    let retrival_erg_len = retrival_erg.len();
    let precision = if retrival_erg_len == 0 {
        0f64
    } else {
        true_positives.len() as f64 / retrival_erg_len as f64
    };

    let recall = if retrival_erg_len == 0 {
        0f64
    } else {
        true_positives.len() as f64 / reference.len() as f64
    };

    (precision, recall)
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

    pub fn fitness(&self, swarm: &mut pso::Swarm){
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
                    calc_precision_recall(&conex_res, self.reference_words);

                // write fitness fn
                p[o.f] = (precision + recall).abs() / 2.0 - (precision - recall).abs() / 3.0;
                // write precision and recall
                p[o.l.0] = precision;
                p[o.l.0+1] = recall;

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
  
    let mut swarm = pso::Swarm::new(
       num_particles, 
       6,
       position_bounds,
       (-1.0, 2.0),
       2,
       &|swarm: &mut pso::Swarm| {
           fitness.fitness(swarm);
       },
       &|_i: usize, swarm: &mut pso::Swarm| {
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
        &|swarm: &mut pso::Swarm| {
            fitness.fitness(swarm);
        },
        &|_i: usize, swarm: &mut pso::Swarm| {
            info!("{} of {}", _i, iterations);
            append_swarm_to_file(swarm, out_file);
        });
}
