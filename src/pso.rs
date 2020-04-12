use ndarray::prelude::*;
use rand::distributions::{Distribution, Uniform};
use rand::Rng;
use log::info;

pub type Bounds = Array<f64, Ix2>;
pub type Particle = Array<f64, Ix1>;
pub type Particles = Array<f64, Ix2>;

#[derive(Debug)]
pub struct Offsets {
    pub p: (usize, usize),
    pub v: (usize, usize),
    pub pb: (usize, usize),
    pub l: (usize, usize),
    pub f: usize, 
    pub pbf: usize
}

pub struct HyperParams {
    pub learning_cognitive: f64,
    pub learning_social: f64,
    pub inertia: f64
}

pub struct Swarm {
    pub pos_dim: usize, 
    pub pos_bounds: Bounds,
    pub fitness_bounds: (f64, f64),
    pub payload_dim: usize,
    pub o: Offsets,
    pub leader: usize,
    pub particles: Particles
}


impl Swarm {

    pub fn new(
        size: usize,
        pos_dim: usize,
        pos_bounds: Bounds, 
        fitness_bounds: (f64, f64),
        // fitness function tmp storage, for plotting
        payload_dim: usize, 
        f: &dyn Fn(&mut Swarm),
        c: &dyn Fn(usize, &mut Swarm)) -> Swarm {

        // curr_pos + velocity + best_pos + curr_fitness + best_fitness + payload
        let dim = pos_dim * 3 + 2 + payload_dim;

        // init zero matrix
        let mut ps: Array<f64, Ix2> = Array::zeros((size, dim));

        let o = Offsets { 
            // position
            p: (0, pos_dim),               
            // fitness
            f: pos_dim,
            //velocity
            v: (pos_dim + 1, 2 * pos_dim + 1),    
            // personal best 
            pb: (2 * pos_dim + 1, 3 * pos_dim + 1),  
            // personal best fitness
            pbf: 3 * pos_dim + 1,
            // payload
            l: (3 * pos_dim + 2, dim),

        };

        info!("offsets: {:?}", o);

        // needed for random particle positioning
        let mut rng = rand::thread_rng();

        // initialize random positions
        azip![(
            mut a_col in ps.slice_mut(
                s![.., o.p.0..o.p.1])
            .gencolumns_mut(),

            b_row in pos_bounds.genrows()) {

            let d = Uniform::new_inclusive(b_row[0], b_row[1]);
            a_col.iter_mut().for_each(|x| *x = d.sample(&mut rng));
        }];

        // initialize personal best fitness with lowest fitness bound
        ps.slice_mut(s![.., o.pbf])
            .iter_mut().for_each(|f| *f = fitness_bounds.0);

            let mut swarm = Swarm {
                pos_dim, pos_bounds, fitness_bounds, payload_dim, 
                o,
                leader: 0,
                particles: ps
            };

            f(&mut swarm);

            swarm.set_personal_bests();
            swarm.find_and_set_leader();

            c(0, &mut swarm);

            swarm
    }

    fn set_personal_bests(&mut self){

        let o = &self.o;
        self.particles.axis_iter_mut(Axis(0)).for_each(|mut row| {
            let curr_f = row[o.f];
            if curr_f >= row[o.pbf] {
                let row_ = row.slice(s![o.p.0 .. o.p.1 + 1]).into_owned();

                // copy position + fitness to personal best 
                row.slice_mut(s![o.pb.0 .. o.pb.1 + 1])
                    .iter_mut().enumerate().for_each(|(i, x)| *x = row_[i]);
            }
        });
    }

    fn find_and_set_leader(&mut self) {
        let (i, _) = self.particles.column(self.o.f).iter().enumerate()
            .fold((0, self.fitness_bounds.0), |(curr_i, curr_max_f), (i, f)| 
                if *f >= curr_max_f {
                    (i, *f)
                } else {
                    (curr_i, curr_max_f)
                });

        self.leader = i;
    }

    pub fn fly(
        &mut self,
        n: usize,
        hp: &HyperParams,
        f: &dyn Fn(&mut Swarm),
        c: &dyn Fn(usize, &mut Swarm)) {

        (0..n).for_each(|i| {
            self.update_position(hp);
            f(self);
            self.set_personal_bests();
            self.find_and_set_leader();
            c(i+1, self);
        });
    }

    fn update_position(&mut self, hp: &HyperParams) {

        let mut rng = rand::thread_rng();

        let o = &self.o;
        let leader = self.particles.row(self.leader).into_owned();
        let pos_dim = self.pos_dim;
        let pos_bounds = &self.pos_bounds;

        self.particles.axis_iter_mut(Axis(0)).for_each(|mut row| {
            (0..pos_dim).for_each(|i| {

                let c1r1:f64 = 
                    hp.learning_cognitive * rng.gen::<f64>();
                let c2r2:f64 = 
                    hp.learning_social * rng.gen::<f64>();

                let v_i = o.v.0 + i;
                let p_i = o.p.0 + i;
                let pb_i = o.pb.0 + i;

                row[v_i] = hp.inertia * row[v_i]
                    + c1r1 * (row[pb_i] - row[p_i])  
                    + c2r2 * (leader[p_i] - row[p_i]);

                row[p_i] += row[v_i];

                let pb = pos_bounds.row(i);
                row[p_i] = pb[0].max(row[p_i]); 
                row[p_i] = pb[1].min(row[p_i]);
            });
        });

    }
}

// use rand::Rng;
// use rand::rngs::ThreadRng;
// use rand::distributions::{Distribution, Uniform};
// use log::info;
// use std::collections::HashMap;

// pub type ParticleT = Vec<f64>;
// pub type Bound = (f64, f64);

// pub trait FitnessFn: Sync + Sized {
//     fn calc_fitness(&self, particle: &mut ParticleT);
// }

// pub struct Swarm<'a, F: FitnessFn> {
//     pub learning_cognitive: f64,
//     pub learning_social: f64,   
//     pub inertia: f64,
//     pub position_dim: usize,
//     pub positions_bounds: Vec<Bound>,
//     pub fitness_bounds: Bound,
//     pub fitness_fn: &'a F,
//     pub leader: usize,
//     pub particles: 
// }

// impl<F: FitnessFn> Swarm<'_, F> {

//     pub fn generate_random_particles(
//         num_particles: usize, 
//         position_distributions: &[Uniform<f64>],
//         position_dim: usize,
//         fitness_bounds: Bound, 
//         rng: &mut ThreadRng) -> Vec<ParticleT> {


//         (0..num_particles).map(|id| {

//             // randomize position
//             let mut curr: ParticleT = position_distributions.iter().map(|d| {
//                 d.sample(rng)
//             }).collect();

//             // initialize velocity
//             curr.append(&mut vec![0f64; position_dim]);

//             // initialize fitness with lower fitness bound
//             curr.push(fitness_bounds.0);
            
//             Particle::new(id, curr, fitness_bounds.0);

//         }).collect();


//         // let mut particles: Vec<Particle> = Vec::with_capacity(num_particles);

//         // let dim_position = uniform_distributions.len(); 
        
//         // for id in 0..num_particles {

//         //     let initial_pos = uniform_distributions.iter().map(|d| {
//         //         d.sample(rng)
//         //     }).collect();
            
//         //     let particle = Particle::new(id, initial_pos, fitness_bounds.0);
//         //     particles.push(particle); 
//         // }

//         // particles
//     }

//     pub fn new<'a, T: FitnessFn>(
//         num_particles: usize,
//         learning_cognitive: f64,
//         learning_social: f64,
//         inertia: f64,
//         position_bounds: Vec<Bound>,
//         fitness_bounds: Bound,
//         fitness_fn: &'a T,
//         on_iteration: &dyn Fn(usize, Vec<(usize, Vec<f64>)>, &Swarm<T>) -> ()) -> Swarm<'a, T> {

//         // get random generator
//         let mut rng = rand::thread_rng();

//         // dimensions
//         let position_dim: usize = position_bounds.len();

//         // for each position dimension build a Uniform 
//         // distribution w.r.t. position bounds
//         let uniform_distribution: Vec<Uniform<f64>> = position_bounds.iter()
//                 .map(|(l, h)| Uniform::new_inclusive(l, h))
//                 .collect();

//         // generate some particles (fitness is evaluated)
//         let particles = Swarm::<T>::generate_random_particles(
//             num_particles, &uniform_distribution, fitness_bounds, &mut rng);

//         let mut swarm = Swarm::<T> {
//             learning_cognitive,
//             learning_social,
//             inertia,
//             position_bounds,
//             fitness_bounds,
//             fitness_fn,
//             leader: (fitness_bounds.0, vec![0f64; position_dim]), 
//             particles,
//         };

//         let payloads = swarm.update_particle_fitness();
//         swarm.select_new_leaders();
//         on_iteration(0, payloads, &swarm);
//         swarm
//     }

//     // TODO slow with too much mem copy
//     pub fn select_new_leaders(&mut self){

//         self.leader = self.particles.iter()
//             .fold((self.fitness_bounds.0, vec![0f64; self.position_bounds.len()]), 
//                 |(cf, cp), p|
//                 if p.fitness >= cf {
//                     (p.fitness, p.position.clone())
//                 } else { (cf, cp) });

//         info!("leader_fitness: {}", self.leader.0);
//     }
    
//     pub fn update_particles(&mut self){

//         let mut rng = rand::thread_rng();

//         for particle in self.particles.iter_mut() {
//             let mut velocity = vec![0f64; self.position_bounds.len()];
//             for (i, x_i) in particle.position.iter_mut().enumerate() {

//                 let c1r1:f64 = 
//                     self.learning_cognitive * rng.gen::<f64>();
//                 let c2r2:f64 = 
//                     self.learning_social * rng.gen::<f64>();

//                 velocity[i] = self.inertia * particle.velocity[i]
//                     + c1r1 * (particle.best_position[i] - *x_i)  
//                     + c2r2 * (self.leader.1[i] - *x_i);

//                 *x_i += velocity[i];

//                 let (l, h) = self.position_bounds[i];
//                 *x_i = l.max(*x_i); 
//                 *x_i = h.min(*x_i);
//             }
//             particle.velocity = velocity;

//         }

//     }

//     pub fn update_particle_fitness(&mut self) -> Vec<(usize, Vec<f64>)>{

//         let particles: HashMap<usize, (Fitness, Vec<f64>)> = 
//             self.particles.par_iter()
//             .map(|particle: &Particle| {
//                 let fitness_and_payload = 
//                     self.fitness_fn.calc_fitness(&particle.position);

//                 (particle.id, fitness_and_payload) 

//         }).collect::<Vec<(usize, (Fitness, Vec<f64>))>>()
//         .iter().cloned().collect();

//         // info!("{:?}", particles);

//         let mut payloads: Vec<(usize, Vec<f64>)> = Vec::with_capacity(particles.len());

//         self.particles.iter_mut().for_each(|particle: &mut Particle| 
           
//             if let Some((new_fitness, payload)) = particles.get(&particle.id) {

//                 if new_fitness >= &particle.best_fitness {
//                     particle.best_position = particle.position.clone();
//                     particle.best_fitness = *new_fitness;
//                 }
//                 particle.fitness = *new_fitness;

//                 payloads.push((particle.id, payload.to_vec()));

//             } else {
//                 panic!("No new fitness for particle id found!");
//             });

//         payloads

//     }

//     pub fn fly(&mut self, 
//         iterations: usize,
//         on_iteration: &dyn Fn(usize, Vec<(usize, Vec<f64>)>, &Swarm<F>) -> ()) {

//         for i in 1..iterations+1 {
//             info!("iteration {} of {}", i, iterations - 1);
//             self.update_particles();
//             let payload = self.update_particle_fitness();
//             self.select_new_leaders();
//             on_iteration(i, payload, self);
//         }

//     }
// }

