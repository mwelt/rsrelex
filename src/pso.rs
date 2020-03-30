use rand::Rng;
use rand::distributions::{Distribution, Uniform};

pub struct Swarm {
    pub learning_cognitive: f64,
    pub learning_social: f64,   
    pub inertia: f64,
    pub best: Vec<f64>,
    pub best_fitness: f64,
    pub bounds: Vec<(f64, f64)>,
    pub particles: Vec<Particle>,
    pub fitness_fn: fn(&[f64]) -> f64
}

impl ToString for Swarm {
    fn to_string(&self) -> String {
        [
            "Swarm:",
            "\nlearning_cognitive:", &self.learning_cognitive.to_string(),
            "\nlearning_social:", &self.learning_social.to_string(),
            "\ninertia:", &self.inertia.to_string(),
            "\nbest:", &self.best.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "),
            "\nbeest_fitness:", &self.best_fitness.to_string(),
        ].join(" ")
    }
}

impl Swarm {
    pub fn new(
        num_particles: usize,
        bounds: Vec<(f64, f64)>,
        learning_cognitive: f64,
        learning_social: f64,
        inertia: f64,
        fitness_fn: fn(&[f64]) -> f64) -> Swarm {

        let n: usize = bounds.len();

        let uniform_distribution: Vec<Uniform<f64>> = bounds.iter()
                .map(|(l, h)| Uniform::new_inclusive(l, h))
                .collect();

        let mut particles: Vec<Particle> = Vec::with_capacity(num_particles);

        for _ in 0..num_particles {
            particles.push(Particle::new_rnd(&uniform_distribution)); 
        }

        Swarm {
            learning_cognitive,
            learning_social,
            inertia,
            best: vec![0f64; n],
            best_fitness: std::f64::MIN,
            bounds,
            particles,
            fitness_fn
        }
    }

    pub fn update_particles(&mut self){

        let mut rng = rand::thread_rng();

        let dim = self.best.len();

        let mut best_in_run_fitness = std::f64::MIN;
        let mut best_in_run_position = vec![0f64; dim]; 

        // update particles
        for particle in self.particles.iter_mut() {
            let mut velocity = vec![0f64; dim];
            for (i, x_i) in particle.pos.iter_mut().enumerate() {
                let c1r1:f64 = 
                    self.learning_cognitive * rng.gen::<f64>();
                let c2r2:f64 = 
                    self.learning_social * rng.gen::<f64>();

                velocity[i] = self.inertia * particle.velocity[i]
                    + c1r1 * (particle.best[i] - *x_i)  
                    + c2r2 * (self.best[i] - *x_i);

                *x_i += velocity[i];
            }
            particle.velocity = velocity;

            // calc particle fitness 
            let particle_fitness = (self.fitness_fn)(&particle.pos);
            particle.update_best(particle_fitness);

            // update best in run
            if best_in_run_fitness < particle_fitness {
                best_in_run_position = particle.pos.clone();
                best_in_run_fitness = particle_fitness;
            }
        }

        // update swarm best
        if self.best_fitness < best_in_run_fitness {
            self.best = best_in_run_position;
            self.best_fitness = best_in_run_fitness;
        }
    }
}

#[derive(Debug)]
pub struct Particle {
    pub best: Vec<f64>,
    pub best_fitness: f64,
    pub pos: Vec<f64>,
    pub velocity: Vec<f64> 
    
}

impl Particle {

    pub fn new_rnd(distribution: &[Uniform<f64>]) -> Particle {
        let len = distribution.len();
        let mut rng = rand::thread_rng();
        Particle::new(vec![0f64; len].iter().enumerate()
            .map(|(i, _)| distribution[i].sample(&mut rng)).collect())
    }

    pub fn new(initial_pos: Vec<f64>) -> Particle {
        let l = initial_pos.len();
        Particle {
            best: initial_pos.clone(),
            best_fitness: std::f64::MIN,
            pos: initial_pos,
            velocity: vec![0f64; l] 
        }
        
    }

    pub fn update_best(&mut self, fitness: f64) {
        if self.best_fitness < fitness {
            self.best_fitness = fitness;
            self.best = self.pos.clone();
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

pub fn pareto_front(xs: &[Vec<f64>], directions: &[bool]) -> Vec<usize> {
    let len = xs.len();
    let mut pareto_front = Vec::new();
    
    for i in 0..len {
        let mut is_dominated = false;
        let p = &xs[i];
        for j in i+1..len {
            if dominates(&xs[j], &p, directions) {
                is_dominated = true;
                break;
            }
        }

        if !is_dominated {
            pareto_front.push(i);
        }
    }

    pareto_front
}

