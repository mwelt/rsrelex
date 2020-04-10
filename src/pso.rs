use rand::seq::SliceRandom;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;
use log::info;
use std::collections::HashMap;

pub type Position = Vec<f64>;
pub type Velocity = Vec<f64>;
pub type Fitness = f64;
pub type ParetoDirection = bool;
pub type Bound = (f64, f64);

pub trait FitnessFn: Sync + Sized {
    fn calc_fitness(&self, pos: &Position) -> Fitness;
}

pub struct Swarm<'a, F: FitnessFn + Sized> {
    pub learning_cognitive: f64,
    pub learning_social: f64,   
    pub inertia: f64,
    pub position_bounds: Vec<Bound>,
    pub fitness_bounds: Bound,
    pub fitness_fn: &'a F,
    // pub leaders: Vec<Leader>,
    pub leader: (Fitness, Position),
    pub particles: Vec<Particle>
}

impl<F: FitnessFn> Swarm<'_, F> {

    pub fn generate_random_particles(
        num_particles: usize, 
        uniform_distributions: &[Uniform<f64>],
        fitness_bounds: Bound,
        rng: &mut ThreadRng) -> Vec<Particle> {

        let mut particles: Vec<Particle> = Vec::with_capacity(num_particles);

        let dim_position = uniform_distributions.len(); 
        
        for id in 0..num_particles {

            let initial_pos = uniform_distributions.iter().map(|d| {
                d.sample(rng)
            }).collect();
            
            let particle = Particle::new(id, initial_pos, fitness_bounds.0);
            particles.push(particle); 
        }

        particles
    }

    pub fn new<'a, T: FitnessFn>(
        num_particles: usize,
        learning_cognitive: f64,
        learning_social: f64,
        inertia: f64,
        position_bounds: Vec<Bound>,
        fitness_bounds: Bound,
        fitness_fn: &'a T) -> Swarm<'a, T> {

        // get random generator
        let mut rng = rand::thread_rng();

        // dimensions
        let position_dim: usize = position_bounds.len();

        // for each position dimension build a Uniform 
        // distribution w.r.t. position bounds
        let uniform_distribution: Vec<Uniform<f64>> = position_bounds.iter()
                .map(|(l, h)| Uniform::new_inclusive(l, h))
                .collect();

        // generate some particles (fitness is evaluated)
        let particles = Swarm::<T>::generate_random_particles(
            num_particles, &uniform_distribution, fitness_bounds, &mut rng);

        let mut swarm = Swarm::<T> {
            learning_cognitive,
            learning_social,
            inertia,
            position_bounds,
            fitness_bounds,
            fitness_fn,
            leader: (fitness_bounds.0, vec![0f64; position_dim]), 
            particles,
        };

        swarm.update_particle_fitness();
        swarm.select_new_leaders();
        swarm
    }

    // TODO slow with too much mem copy
    pub fn select_new_leaders(&mut self){

        self.leader = self.particles.iter()
            .fold((self.fitness_bounds.0, vec![0f64; self.position_bounds.len()]), 
                |(cf, cp), p|
                if p.fitness >= cf {
                    (p.fitness, p.position.clone())
                } else { (cf, cp) });

        info!("leader_fitness: {}", self.leader.0);
    }
    
    pub fn update_particles(&mut self){

        let mut rng = rand::thread_rng();

        for particle in self.particles.iter_mut() {
            let mut velocity = vec![0f64; self.position_bounds.len()];
            for (i, x_i) in particle.position.iter_mut().enumerate() {

                let c1r1:f64 = 
                    self.learning_cognitive * rng.gen::<f64>();
                let c2r2:f64 = 
                    self.learning_social * rng.gen::<f64>();

                velocity[i] = self.inertia * particle.velocity[i]
                    + c1r1 * (particle.best_position[i] - *x_i)  
                    + c2r2 * (self.leader.1[i] - *x_i);

                *x_i += velocity[i];

                let (l, h) = self.position_bounds[i];
                *x_i = l.max(*x_i); 
                *x_i = h.min(*x_i);
            }
            particle.velocity = velocity;

        }

    }

    pub fn update_particle_fitness(&mut self){

        let particles: HashMap<usize, Fitness> = 
            self.particles.par_iter()
            .map(|particle: &Particle| {
                let fitness = self.fitness_fn.calc_fitness(&particle.position);

                (particle.id, fitness) 

        }).collect::<Vec<(usize, Fitness)>>()
        .iter().cloned().collect();

        // info!("{:?}", particles);

        self.particles.iter_mut().for_each(|particle: &mut Particle| 
           
            if let Some(new_fitness) = particles.get(&particle.id) {

                if new_fitness >= &particle.best_fitness {
                    particle.best_position = particle.position.clone();
                    particle.best_fitness = *new_fitness;
                }
                particle.fitness = *new_fitness;
            } else {
                panic!("No new fitness for particle id found!");
            });

    }

    pub fn fly(&mut self, 
        iterations: usize,
        on_iteration: &dyn Fn(usize, &Swarm<F>) -> ()) {

        for i in 0..iterations {
            info!("iteration {} of {}", i, iterations - 1);
            self.update_particles();
            self.update_particle_fitness();
            self.select_new_leaders();
            on_iteration(i, self);
        }

    }
}

#[derive(Debug)]
pub struct Particle {
    pub id: usize,
    pub best_position: Position,
    pub best_fitness: Fitness,
    pub position: Position,
    pub fitness: Fitness,
    pub velocity: Velocity 
    
}

impl Particle {

    pub fn new(id: usize, initial_pos: Position, initial_fitness: Fitness) -> Particle {
        let l = initial_pos.len();
        Particle {
            id,
            best_position: initial_pos.clone(),
            best_fitness: initial_fitness.clone(), 
            fitness: initial_fitness,
            position: initial_pos,
            velocity: vec![0f64; l] 
        }
        
    }

}
