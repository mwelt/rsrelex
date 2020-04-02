use rand::seq::SliceRandom;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, Uniform};

pub type Position = Vec<f64>;
pub type Velocity = Vec<f64>;
pub type Fitness = Vec<f64>;
pub type Quality = f64;
pub type ParetoDirection = bool;
pub type Bound = (f64, f64);
pub type FitnessFn = fn(&Position) -> Fitness;

#[derive(Clone)]
pub struct Leader {
    pub position: Position, 
    pub fitness: Fitness, 
    pub quality: Quality,
    pub rank: usize
}

pub struct Swarm {
    pub learning_cognitive: f64,
    pub learning_social: f64,   
    pub inertia: f64,
    pub position_dim: usize,
    pub position_bounds: Vec<Bound>,
    pub fitness_bounds: Vec<Bound>,
    pub fitness_dim: usize,
    pub fitness_pareto_directions: Vec<ParetoDirection>,
    pub fitness_fn: FitnessFn,
    pub leaders: Vec<Leader>,
    pub particles: Vec<Particle>
}

impl ToString for Swarm {
    fn to_string(&self) -> String {
        [
            "Swarm:",
            "\nlearning_cognitive:", &self.learning_cognitive.to_string(),
            "\nlearning_social:", &self.learning_social.to_string(),
            "\ninertia:", &self.inertia.to_string(),
        ].join(" ")
    }
}

impl Swarm {

    pub fn generate_random_particles(
        num_particles: usize, 
        uniform_distributions: &Vec<Uniform<f64>>,
        fitness_fn: FitnessFn,
        rng: &mut ThreadRng) -> Vec<Particle> {

        let mut particles: Vec<Particle> = Vec::with_capacity(num_particles);

        let dim_position = uniform_distributions.len();

        for _ in 0..num_particles {

            let mut initial_pos = Vec::with_capacity(dim_position);
            for i in 0..dim_position {
                initial_pos.push(uniform_distributions[i].sample(rng)); 
            }
            let initial_fitness = fitness_fn(&initial_pos);
            
            let particle = Particle::new(initial_pos, initial_fitness);
            particles.push(particle); 
        }

        particles
    }

    pub fn new(
        num_particles: usize,
        learning_cognitive: f64,
        learning_social: f64,
        inertia: f64,
        position_bounds: Vec<Bound>,
        fitness_bounds: Vec<Bound>,
        fitness_pareto_directions: Vec<ParetoDirection>,
        fitness_fn: fn(&Position) -> Fitness) -> Swarm {

        // get random generator
        let mut rng = rand::thread_rng();

        // dimensions
        let position_dim: usize = position_bounds.len();
        let fitness_dim: usize = fitness_bounds.len();

        // for each position dimension build a Uniform 
        // distribution w.r.t. position bounds
        let uniform_distribution: Vec<Uniform<f64>> = position_bounds.iter()
                .map(|(l, h)| Uniform::new_inclusive(l, h))
                .collect();

        // generate some particles (fitness is evaluated)
        let particles = Swarm::generate_random_particles(
            num_particles, &uniform_distribution, fitness_fn, &mut rng);

        let mut swarm = Swarm {
            learning_cognitive,
            learning_social,
            inertia,
            position_dim, 
            position_bounds,
            fitness_bounds,
            fitness_dim,
            fitness_pareto_directions,
            fitness_fn,
            leaders: Vec::new(), 
            particles,
        };

        swarm.select_new_leaders();
        swarm.qualify_leaders();
        swarm
    }

    // TODO slow with too much mem copy
    pub fn select_new_leaders(&mut self){

        let mut potential_leaders: Vec<Leader> = Vec::with_capacity(
            self.leaders.len() + self.particles.len());

        self.leaders.iter().for_each(|l| potential_leaders.push(l.clone()));
        self.particles.iter().for_each(|p| potential_leaders.push(
                Leader {
                    position: p.position.clone(),
                    fitness: p.fitness.clone(),
                    quality: 0f64,
                    rank: 0usize
                }));
        
        let fitness_values: Vec<&Fitness> = potential_leaders.iter()
            .map(|l| &l.fitness).collect();

        let pareto_idxs = pareto_front(&fitness_values,
            &self.fitness_pareto_directions);

        self.leaders.clear();
        pareto_idxs.iter().for_each(|i| 
            self.leaders.push(potential_leaders[*i].clone()));

    }

    
    // Diversity Management via crowding-distance 
    pub fn qualify_leaders(&mut self) {

        let len = self.leaders.len();

        // first initialize the distance vector
        // can not use self.leaders[i].quality directly, bc.
        // borrowing the fitness values already 
        let mut distances: Vec<f64> = Vec::with_capacity(self.leaders.len());

        let mut leaders_tmp: Vec<(usize, &Fitness)> = 
            self.leaders.iter().enumerate()
            .map(|(i, l)| (i, &l.fitness)).collect();

        for (d, (l, h)) in self.fitness_bounds.iter().enumerate() {

            let bound_dist = h - l;

            leaders_tmp.sort_unstable_by(|(_, l1), (_, l2)| {
                if let Some(ordering) = 
                    l1[d].partial_cmp(&l2[d]) {
                        ordering
                    } else {
                        std::cmp::Ordering::Equal
                }
            });

            let (fst, _) = leaders_tmp[0];
            let (lst, _) = leaders_tmp[len - 1];

            distances[fst] = std::f64::MAX;
            distances[lst] = std::f64::MAX;

            // iterate over second to second last
            for i in 1..(len - 2) {
                // index of current leader in distances vec
                let (j, _) = leaders_tmp[i];
                // prev and next in sorted by d'th dimension
                let (_, leader_prev) = leaders_tmp[i-1];
                let (_, leader_next) = leaders_tmp[i+1]; 

                distances[j] += (leader_next[d] - leader_prev[d]) / bound_dist;  
                
            }

        }

        for (i, leader) in self.leaders.iter_mut().enumerate() {
            leader.quality = distances[i];
        }

        //sort the leaders 
        self.leaders.sort_unstable_by(|l1, l2| {
            if let Some(ordering) = 
                l1.quality.partial_cmp(&l2.quality) {
                    ordering
                } else {
                    std::cmp::Ordering::Equal
            }
        });

    }

    pub fn select_next_leader<'a>(leaders: &'a [Leader], 
        rng: &mut ThreadRng) -> &'a Vec<f64> {
        
        // TODO choose leader by qualification 
        // the better the qualification the more likely 
        // shall be the selection ... see initialzing for 
        // k-means 
        let rng_leader = leaders.choose(rng)
            .expect("unable to choose random next leader");
        &rng_leader.position  
    }

    pub fn update_particles(&mut self){

        let mut rng = rand::thread_rng();

        // TODO parallelize (select leader upfront)
        // update particles
        for particle in self.particles.iter_mut() {
            let mut velocity = vec![0f64; self.position_dim];
            let particle_leader = 
                Swarm::select_next_leader(&self.leaders, &mut rng);
            for (i, x_i) in particle.position.iter_mut().enumerate() {

                let c1r1:f64 = 
                    self.learning_cognitive * rng.gen::<f64>();
                let c2r2:f64 = 
                    self.learning_social * rng.gen::<f64>();

                velocity[i] = self.inertia * particle.velocity[i]
                    + c1r1 * (particle.best_position[i] - *x_i)  
                    + c2r2 * (particle_leader[i] - *x_i);

                *x_i += velocity[i];

                let (l, h) = self.position_bounds[i];
                *x_i = l.max(*x_i); 
                *x_i = h.min(*x_i);
            }
            particle.velocity = velocity;

            let fitness = (self.fitness_fn)(&particle.position);
            // this is a little tricky. The personal best is 
            // only updated if the best stille dominates the 
            // new one, if not the new position is taken either way
            if !dominates(&particle.best_fitness, &fitness, 
                &self.fitness_pareto_directions) {

                particle.best_position = particle.position.clone();
                particle.best_fitness = fitness;
            }
        }

    }

    pub fn fly(&mut self, 
        iterations: usize,
        on_iteration: fn(i: usize, &Swarm) -> ()) {

        for i in 0..iterations {
            self.update_particles();
            self.select_new_leaders();
            self.qualify_leaders();
            on_iteration(i, self);
        }

    }
}

#[derive(Debug)]
pub struct Particle {
    pub best_position: Position,
    pub best_fitness: Fitness,
    pub position: Position,
    pub fitness: Fitness,
    pub velocity: Velocity 
    
}

impl Particle {

    pub fn new(initial_pos: Position, initial_fitness: Fitness) -> Particle {
        let l = initial_pos.len();
        Particle {
            best_position: initial_pos.clone(),
            best_fitness: initial_fitness.clone(), 
            fitness: initial_fitness,
            position: initial_pos,
            velocity: vec![0f64; l] 
        }
        
    }

}

/**
 * Checks if x dominates y in pareto optimal sense. 
 */
pub fn dominates(x: &[f64], y: &[f64], directions: &[bool]) -> bool {
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

pub fn pareto_front(xs: &[&Fitness], directions: &[ParetoDirection]) -> Vec<usize> {
    let mut pareto_front: Vec<usize> = Vec::new();

    for (i, x) in xs.iter().enumerate() {
        let mut is_dominated = false;
        for y in xs.iter() {
            if dominates(y, x, directions){
                is_dominated = true;
                break;
            }
        }
        if ! is_dominated {
            pareto_front.push(i);
        }
    }
    // for i in 0..len {
    //     let mut is_dominated = false;
    //     let p = &xs[i];
    //     for j in 0..len {
    //         if dominates(&xs[j], &p, directions) {
    //             is_dominated = true;
    //             break;
    //         }
    //     }

    //     if !is_dominated {
    //         pareto_front.push(i);
    //     }
    // }

    pareto_front
}

// TODO quadratic cubic distance?
pub fn distance(x: &[f64], y: &[f64]) -> f64 {

    let mut sum = 0f64;
    for i in 0..x.len() {
        let d = y[i] - x[i];
        sum += d*d; 
    }

    sum.sqrt()
}
