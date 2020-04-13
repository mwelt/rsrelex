use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use std::io::Write;
use std::fs::OpenOptions;

fn append_swarm_to_file(s: &mopso::Swarm, f: &str){

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
}
