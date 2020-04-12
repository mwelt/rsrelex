use super::pso::{HyperParams, Bounds, Particles};
use ndarray::prelude::*;
use ndarray::stack;
use log::info;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;
// use rayon::prelude::*;
// use std::collections::HashMap;

#[derive(Debug)]
pub struct Offsets {
    pub p: (usize, usize),
    pub v: (usize, usize),
    pub pb: (usize, usize),
    pub l: (usize, usize),
    pub f: (usize, usize),
    pub pbf: (usize, usize)
}

// pub struct Leader {
//     i: usize, 
//     r: usize, 
//     d: f64
// }

pub struct Swarm {
    pub pos_dim: usize,
    pub pos_bounds: Bounds,
    pub fitness_dim: usize,
    pub fitness_bounds: Bounds,
    pub pareto_directions: Array<bool, Ix1>,
    pub payload_dim: usize,
    pub o: Offsets,
    pub leaders: Array<f64, Ix2>,
    pub leaders_pos_by_rank: Vec<(usize, Array1<f64>)>,
    pub particles: Particles
}

impl Swarm {
    pub fn new(
        size: usize,
        pos_dim: usize,
        pos_bounds: Bounds,
        fitness_dim: usize,
        fitness_bounds: Bounds,
        pareto_directions: Array<bool, Ix1>,
        payload_dim: usize,
        f: &dyn Fn(&mut Swarm),
        c: &dyn Fn(usize, &mut Swarm)
    ) -> Swarm {

        let dim = pos_dim * 3 + 2 * fitness_dim + payload_dim;

        let mut ps: Array<f64, Ix2> = Array::zeros((size, dim));

        let o = Offsets {
            // position
            p: (0, pos_dim),
            // fitness
            f: (pos_dim, pos_dim + fitness_dim),
            // velocity
            v: (pos_dim + fitness_dim, 2 * pos_dim + fitness_dim),
            // personal best
            pb: (2 * pos_dim + fitness_dim, 3 * pos_dim + fitness_dim),
            // personal best fitness
            pbf: (3 * pos_dim + fitness_dim, 3 * pos_dim + 2 * fitness_dim),
            // payload
            l: (3 * pos_dim + 2 * fitness_dim, 3 * pos_dim + 2 * fitness_dim + payload_dim)
        };

        info!("offsets: {:?}", o);

        let mut rng = rand::thread_rng();

        // init random positions
        azip![(
            mut col in ps.slice_mut(
                s![.., o.p.0..o.p.1])
            .gencolumns_mut(),

            row in pos_bounds.genrows()) {

            let d = Uniform::new_inclusive(row[0], row[1]);
            col.iter_mut().for_each(|x| *x = d.sample(&mut rng));
        }];

        // init personal best fitness with lower fitness bounds
        azip![(
            mut col in ps.slice_mut(
                s![.., o.pbf.0..o.pbf.1])
            .gencolumns_mut(),

            row in fitness_bounds.genrows()) {

            col.iter_mut().for_each(|x| *x = row[0]);
        }];

        let mut swarm = Swarm {
            pos_dim,
            pos_bounds,
            fitness_dim,
            fitness_bounds,
            pareto_directions,
            payload_dim,
            o,
            leaders: Array::zeros((0, dim)),
            leaders_pos_by_rank: Vec::new(),
            particles: ps
        };

        f(&mut swarm);

        swarm.set_personal_bests();
        swarm.find_and_set_leaders();
        swarm.calc_crowding_distances();

        c(0, &mut swarm);

        swarm
    }

    fn set_personal_bests(&mut self) {
        let o = &self.o;
        let pareto_directions = &self.pareto_directions.view();

        self.particles.axis_iter_mut(Axis(0)).for_each(|mut row| {
            let f = row.slice(s![o.f.0..o.f.1]);
            let pbf = row.slice(s![o.pbf.0..o.pbf.1]);
               if !dominates(&pbf, &f, pareto_directions) {
                   (0..o.f.1).for_each(|i| {
                       let i_ = i + o.pb.0; 
                       row[i_] = row[i];
                   });
               } 
        });
    }

    fn find_and_set_leaders(&mut self) {

        let pseudo_leaders = stack![Axis(0), self.leaders, self.particles];

        let fitnesss = pseudo_leaders.slice(s![.., self.o.f.0..self.o.f.1]); 
       
        let pareto_idx = pareto_front(&fitnesss, &self.pareto_directions.view());

        self.leaders = pseudo_leaders.select(Axis(0), &pareto_idx);
    }

    fn calc_crowding_distances(&mut self) {

        let l = self.leaders.shape()[0];
        let f_slice = self.leaders.slice(s![.., self.o.f.0..self.o.f.1]);
        let mut fitnesss = f_slice 
            .axis_iter(Axis(0)).enumerate()
            .collect::<Vec<(usize, ArrayView1<f64>)>>(); 

        let mut distances: Array1<f64> = Array::zeros(l);

        self.fitness_bounds.axis_iter(Axis(0)).enumerate()
            .for_each(|(d, b)|{

                fitnesss.sort_unstable_by(|(_, f1), (_, f2)| 
                    f64_sort(f1[d], f2[d]));

                let (fst, _) = fitnesss[0];
                let (lst, _) = fitnesss[l - 1];

                distances[fst] = std::f64::MAX;
                distances[lst] = std::f64::MAX;

                let b_dist = b[1] - b[0];

                (1..l-1).for_each(|i| {
                    let (j, _) = fitnesss[i];

                    let (_, prev) = fitnesss[i-1];
                    let (_, next) = fitnesss[i+1];

                    distances[j] += (next[d] - prev[d]) / b_dist;
                });

            });

        // first map distances to their array index aka. 
        // the leaders index
        let mut distances_by_idx = distances.iter().enumerate()
            .collect::<Vec<(usize, &f64)>>();
        // then sort by distance
        distances_by_idx.sort_unstable_by(|(_, d1), (_, d2)| 
            f64_sort(**d1, **d2));
        // finally enumerate again, this time the index denotes
        // the inverse rank 
        // (lowest rank == lowest avg. distance => crowded neighbourhood)
        // this inverse rank is later used to select randomly weighted a 
        // leader for a given particle
        self.leaders_pos_by_rank = distances_by_idx.iter().enumerate()
            .map(|(r, (i, _))| 
                (r + 1, self.leaders.slice(s![*i, self.o.p.0..self.o.p.1]).to_owned()))
            .collect::<Vec<(usize, Array1<f64>)>>(); 

    }

   pub fn select_next_leader<'a>(leaders: &'a [(usize, Array1<f64>)], s: usize,
       rng: &mut ThreadRng) -> &'a Array1<f64> {
       &leaders.choose_weighted(rng, |(r, _)| *r as f64 / s as f64)
           .expect("Can not choose next leader weighted.").1
   }

   fn update_position(&mut self, hp: &HyperParams) {

       let mut rng = rand::thread_rng();

       let o = &self.o;
       let pos_dim = self.pos_dim;
       let pos_bounds = &self.pos_bounds;
       let leader_pos_by_rank = &self.leaders_pos_by_rank;
       let s = (leader_pos_by_rank.len() * (leader_pos_by_rank.len() + 1)) / 2; 

       self.particles.axis_iter_mut(Axis(0)).for_each(|mut row| {
           (0..pos_dim).for_each(|i| {

               let c1r1:f64 = 
                   hp.learning_cognitive * rng.gen::<f64>();
               let c2r2:f64 = 
                   hp.learning_social * rng.gen::<f64>();

               let leader = Swarm::select_next_leader(leader_pos_by_rank, s, &mut rng); 

               let v_i = o.v.0 + i;
               let p_i = o.p.0 + i;
               let pb_i = o.pb.0 + i;

               row[v_i] = hp.inertia * row[v_i]
                   + c1r1 * (row[pb_i] - row[p_i])  
                   + c2r2 * (leader[i] - row[p_i]);

               row[p_i] += row[v_i];

               let pb = pos_bounds.row(i);
               row[p_i] = pb[0].max(row[p_i]); 
               row[p_i] = pb[1].min(row[p_i]);
           });
       });

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
            self.find_and_set_leaders();
            self.calc_crowding_distances();
            c(i+1, self);
        });
    }
}

fn f64_sort(a: f64, b: f64) -> std::cmp::Ordering {
    if let Some(ord) = 
        a.partial_cmp(&b) {
            ord
        } else {
            std::cmp::Ordering::Equal
    }
}

//pub type Position = Vec<f64>;
//pub type Velocity = Vec<f64>;
//pub type Fitness = Vec<f64>;
//pub type ParetoDirection = bool;
//pub type Bound = (f64, f64);
//// pub type FitnessFn = dyn Fn(&Position) -> Fitness;

//pub trait FitnessFn: Sync + Sized {
//    fn calc_fitness(&self, pos: &Position) -> Fitness;
//}

//#[derive(Clone)]
//pub struct Leader {
//    pub position: Position, 
//    pub fitness: Fitness, 
//    pub crowding_distance: f64,
//    pub rank: usize
//}

//pub struct Swarm<'a, F: FitnessFn + Sized> {
//    pub learning_cognitive: f64,
//    pub learning_social: f64,   
//    pub inertia: f64,
//    pub position_dim: usize,
//    pub position_bounds: Vec<Bound>,
//    pub fitness_bounds: Vec<Bound>,
//    pub fitness_dim: usize,
//    pub fitness_pareto_directions: Vec<ParetoDirection>,
//    pub fitness_fn: &'a F,
//    pub leaders: Vec<Leader>,
//    pub rank_sum: usize,
//    pub particles: Vec<Particle>
//}

//impl<F: FitnessFn> ToString for Swarm<'_, F> {
//    fn to_string(&self) -> String {
//        [
//            "Swarm:",
//            "\nlearning_cognitive:", &self.learning_cognitive.to_string(),
//            "\nlearning_social:", &self.learning_social.to_string(),
//            "\ninertia:", &self.inertia.to_string(),
//        ].join(" ")
//    }
//}

//impl<F: FitnessFn> Swarm<'_, F> {

//    pub fn generate_random_particles(
//        num_particles: usize, 
//        uniform_distributions: &[Uniform<f64>],
//        fitness_dim: usize,
//        rng: &mut ThreadRng) -> Vec<Particle> {

//        let mut particles: Vec<Particle> = Vec::with_capacity(num_particles);

//        let dim_position = uniform_distributions.len();

//        for id in 0..num_particles {

//            let mut initial_pos = Vec::with_capacity(dim_position);
//            for d in uniform_distributions.iter() {
//                initial_pos.push(d.sample(rng)); 
//            }
            
//            let particle = Particle::new(id, initial_pos, vec![0f64; fitness_dim]);
//            particles.push(particle); 
//        }

//        particles
//    }

//    pub fn new<'a, T: FitnessFn>(
//        num_particles: usize,
//        learning_cognitive: f64,
//        learning_social: f64,
//        inertia: f64,
//        position_bounds: Vec<Bound>,
//        fitness_bounds: Vec<Bound>,
//        fitness_pareto_directions: Vec<ParetoDirection>,
//        fitness_fn: &'a T) -> Swarm<'a, T> {

//        // get random generator
//        let mut rng = rand::thread_rng();

//        // dimensions
//        let position_dim: usize = position_bounds.len();
//        let fitness_dim: usize = fitness_bounds.len();

//        // for each position dimension build a Uniform 
//        // distribution w.r.t. position bounds
//        let uniform_distribution: Vec<Uniform<f64>> = position_bounds.iter()
//                .map(|(l, h)| Uniform::new_inclusive(l, h))
//                .collect();

//        // generate some particles (fitness is evaluated)
//        let particles = Swarm::<T>::generate_random_particles(
//            num_particles, &uniform_distribution, fitness_dim, &mut rng);

//        let mut swarm = Swarm::<T> {
//            learning_cognitive,
//            learning_social,
//            inertia,
//            position_dim, 
//            position_bounds,
//            fitness_bounds,
//            fitness_dim,
//            fitness_pareto_directions,
//            fitness_fn,
//            leaders: Vec::new(), 
//            rank_sum: 0usize,
//            particles,
//        };

//        swarm.update_particle_fitness();
//        swarm.select_new_leaders();
//        swarm.pareto_crowding_distance();
//        swarm
//    }

//    // TODO slow with too much mem copy
//    pub fn select_new_leaders(&mut self){

//        let mut potential_leaders: Vec<Leader> = Vec::with_capacity(
//            self.leaders.len() + self.particles.len());

//        self.leaders.iter().for_each(|l| potential_leaders.push(l.clone()));
//        self.particles.iter().for_each(|p| potential_leaders.push(
//                Leader {
//                    position: p.position.clone(),
//                    fitness: p.fitness.clone(),
//                    crowding_distance: 0f64,
//                    rank: 0usize
//                }));
        
//        let fitness_values: Vec<&Fitness> = potential_leaders.iter()
//            .map(|l| &l.fitness).collect();

//        let pareto_idxs = pareto_front(&fitness_values,
//            &self.fitness_pareto_directions);

//        self.leaders.clear();
//        pareto_idxs.iter().for_each(|i| 
//            self.leaders.push(potential_leaders[*i].clone()));

//    }
    
//    // Diversity Management via crowding-distance 
//    pub fn pareto_crowding_distance(&mut self) {

//        let len = self.leaders.len();

//        // first initialize the distance vector
//        // can not use self.leaders[i].crowding_distance directly, bc.
//        // borrowing the fitness values already 
//        let mut distances: Vec<f64> = vec![0f64; self.leaders.len()];

//        let mut leaders_tmp: Vec<(usize, &Fitness)> = 
//            self.leaders.iter().enumerate()
//            .map(|(i, l)| (i, &l.fitness)).collect();

//        for (d, (l, h)) in self.fitness_bounds.iter().enumerate() {

//            let bound_dist = h - l;

//            leaders_tmp.sort_unstable_by(|(_, l1), (_, l2)| {
//                if let Some(ordering) = 
//                    l1[d].partial_cmp(&l2[d]) {
//                        ordering
//                    } else {
//                        std::cmp::Ordering::Equal
//                }
//            });

//            let (fst, _) = leaders_tmp[0];
//            let (lst, _) = leaders_tmp[len - 1];

//            distances[fst] = std::f64::MAX;
//            distances[lst] = std::f64::MAX;

//            // iterate over second to second last
//            for i in 1..(len - 1) {
//                // index of current leader in distances vec
//                let (j, _) = leaders_tmp[i];
//                // prev and next in sorted by d'th dimension
//                let (_, leader_prev) = leaders_tmp[i-1];
//                let (_, leader_next) = leaders_tmp[i+1]; 

//                distances[j] += (leader_next[d] - leader_prev[d]) / bound_dist;  
                
//            }

//        }

//        for (i, leader) in self.leaders.iter_mut().enumerate() {
//            leader.crowding_distance = distances[i];
//        }

//        //sort the leaders 
//        self.leaders.sort_unstable_by(|l1, l2| {
//            if let Some(ordering) = 
//                l1.crowding_distance.partial_cmp(&l2.crowding_distance) {
//                    ordering
//                } else {
//                    std::cmp::Ordering::Equal
//            }
//        });

//        // apply rank to the leaders short crowding distance means
//        // crowded neighborhood -> lower rank
//        let mut sum_rk = 0usize;
//        for (i, leader) in self.leaders.iter_mut().enumerate() {
//            // leader.rank = len - i;
//            leader.rank = i + 1;
//            sum_rk += leader.rank;
//        }

//        self.rank_sum = sum_rk;

//    }

//    pub fn select_next_leader<'a>(leaders: &'a [Leader], sum_rk: usize,
//        rng: &mut ThreadRng) -> &'a Vec<f64> {
        
//        // TODO choose leader by qualification 
//        // the better the qualification the more likely 
//        // shall be the selection ... see initialzing for 
//        // k-means 
//        // let rng_leader = leaders.choose(rng)
//        //     .expect("unable to choose random next leader");
//        let rng_leader = leaders.choose_weighted(rng, |l| l.rank as f64 / sum_rk as f64)
//            .expect("Can not choose next leader weighted.");
//        &rng_leader.position  
//    }

//    pub fn update_particles(&mut self){

//        let mut rng = rand::thread_rng();

//        // TODO parallelize (select leader upfront)
//        // update particles
//        for particle in self.particles.iter_mut() {
//            let mut velocity = vec![0f64; self.position_dim];
//            let particle_leader = 
//                Swarm::<F>::select_next_leader(&self.leaders, self.rank_sum, &mut rng);
//            for (i, x_i) in particle.position.iter_mut().enumerate() {

//                let c1r1:f64 = 
//                    self.learning_cognitive * rng.gen::<f64>();
//                let c2r2:f64 = 
//                    self.learning_social * rng.gen::<f64>();

//                velocity[i] = self.inertia * particle.velocity[i]
//                    + c1r1 * (particle.best_position[i] - *x_i)  
//                    + c2r2 * (particle_leader[i] - *x_i);

//                *x_i += velocity[i];

//                let (l, h) = self.position_bounds[i];
//                *x_i = l.max(*x_i); 
//                *x_i = h.min(*x_i);
//            }
//            particle.velocity = velocity;

//        }

//    }

//    pub fn update_particle_fitness(&mut self){

//        let particles: HashMap<usize, Option<(Position, Fitness)>> = 
//            self.particles.par_iter()
//            .map(|particle| {
//                let fitness = self.fitness_fn.calc_fitness(&particle.position);
//                // this is a little tricky. The personal best is 
//                // only updated if the best stille dominates the 
//                // new one, if not the new position is taken either way
//                if !dominates(&particle.best_fitness, &fitness, 
//                    &self.fitness_pareto_directions) {

//                    (particle.id, Some((particle.position.clone(), fitness)))
//                } else {
//                    (particle.id, None)
//                }

//        }).collect::<Vec<(usize, Option<(Position, Fitness)>)>>()
//        .iter().cloned().collect();

//        self.particles.iter_mut().for_each(|particle| 
//            if let Some(Some((p, f))) = particles.get(&particle.id) {
//                particle.best_position = p.clone();
//                particle.best_fitness = f.clone();
//            });

//    }

//    pub fn fly(&mut self, 
//        iterations: usize,
//        on_iteration: &dyn Fn(usize, &Swarm<F>) -> ()) {

//        for i in 0..iterations {
//            info!("iteration {} of {}", i, iterations - 1);
//            self.update_particles();
//            self.update_particle_fitness();
//            self.select_new_leaders();
//            self.pareto_crowding_distance();
//            on_iteration(i, self);
//        }

//    }
//}

//#[derive(Debug)]
//pub struct Particle {
//    pub id: usize,
//    pub best_position: Position,
//    pub best_fitness: Fitness,
//    pub position: Position,
//    pub fitness: Fitness,
//    pub velocity: Velocity 
    
//}

//impl Particle {

//    pub fn new(id: usize, initial_pos: Position, initial_fitness: Fitness) -> Particle {
//        let l = initial_pos.len();
//        Particle {
//            id,
//            best_position: initial_pos.clone(),
//            best_fitness: initial_fitness.clone(), 
//            fitness: initial_fitness,
//            position: initial_pos,
//            velocity: vec![0f64; l] 
//        }
        
//    }

//}

 /**
  * Checks if x dominates y in pareto optimal sense. 
  */
pub fn dominates(x: &ArrayView1<f64>, y: &ArrayView1<f64>, directions: &ArrayView1<bool>) -> bool {
    let mut all_at_least_equal_or_better = true;
    let mut dominating_dimension_found = false;

    for (i, x_i) in x.iter().enumerate() {
        // \exists x_i : x_i > y_i 
        if (directions[i] && *x_i > y[i])
            || (!directions[i] && *x_i < y[i]) {

            dominating_dimension_found = true;
        }

        // \forall x_i : x_i >= y_i
        // negated to ! \exists x_i : x_i < y_i
        if (directions[i] && *x_i < y[i]) 
            || (!directions[i] && *x_i > y[i]) {

            all_at_least_equal_or_better = false;
            // we can short circuit here (negation of \forall)
            break;
        }
    }

    all_at_least_equal_or_better && dominating_dimension_found
}

pub fn pareto_front(xs: &ArrayView2<f64>, directions: &ArrayView1<bool>) -> Vec<usize> {
    let mut pareto_front: Vec<usize> = Vec::new();

    for (i, x) in xs.axis_iter(Axis(0)).enumerate() {
        let mut is_dominated = false;
        for y in xs.axis_iter(Axis(0)) {
            if dominates(&y, &x, directions){
                is_dominated = true;
                break;
            }
        }
        if ! is_dominated {
            pareto_front.push(i);
        }
    }
    pareto_front
}

//// TODO quadratic cubic distance?
//// pub fn distance(x: &[f64], y: &[f64]) -> f64 {

////     let mut sum = 0f64;
////     for i in 0..x.len() {
////         let d = y[i] - x[i];
////         sum += d*d; 
////     }

////     sum.sqrt()
//// }
