use rand::distributions::{Distribution, Uniform};
use rand_distr::Normal;

#[derive(Debug)]
pub struct Swarm {
    pub uniform_distribution: Vec<Uniform<f64>>,
    pub normal_distribution: Vec<Normal<f64>>,
    pub learning_cognitive: f64,
    pub learning_social: f64,   
    pub inertia: f64,
    pub best: Vec<f64>,
    pub bounds: Vec<(f64, f64)>,
    pub particles: Vec<Particle>
}

impl Swarm {
    pub fn new(
        num_particles: usize,
        bounds: Vec<(f64, f64)>,
        learning_cognitive: f64,
        learning_social: f64,
        inertia: f64) -> Swarm {

        let n: usize = bounds.len();

        let uniform_distribution: Vec<Uniform<f64>> = bounds.iter()
                .map(|(l, h)| Uniform::new_inclusive(l, h))
                .collect();

        let mut particles: Vec<Particle> = Vec::with_capacity(num_particles);

        for _ in 0..num_particles {
            particles.push(Particle::new_rnd(n, &uniform_distribution)); 
        }

        Swarm {
            uniform_distribution,
            normal_distribution: bounds.iter()
                .map(|(l, h)| Normal::new(0f64, (h - l).sqrt())
                    .expect("Wasn't able to build normal distribution"))
                .collect(),
            learning_cognitive,
            learning_social,
            inertia,
            best: vec![0f64; n],
            bounds,
            particles
        }
    }

    pub fn update_velocity(&mut self){

        let mut rng = rand::thread_rng();

        for particle in self.particles.iter_mut() {
            let mut velocity = vec![0f64; particle.velocity.len()];
            for (i, x_i) in particle.pos.iter_mut().enumerate() {
                let c1r1:f64 = 
                    self.learning_cognitive * self.normal_distribution[i].sample(&mut rng);
                let c2r2:f64 = 
                    self.learning_social * self.normal_distribution[i].sample(&mut rng);

                velocity[i] = self.inertia * particle.velocity[i]
                    + c1r1 * (particle.p_best[i] - *x_i)  
                    + c2r2 * (self.best[i] - *x_i);

                *x_i += velocity[i];
            }
            particle.velocity = velocity;
        }
    }
}

#[derive(Debug)]
pub struct Particle {
    pub p_best: Vec<f64>,
    pub pos: Vec<f64>,
    pub velocity: Vec<f64> 
}

impl Particle {

    pub fn new_rnd(len: usize, distribution: &[Uniform<f64>]) -> Particle {
        let mut rng = rand::thread_rng();
        Particle::new(vec![0f64; len].iter().enumerate()
            .map(|(i, _)| distribution[i].sample(&mut rng)).collect())
    }

    pub fn new(initial_pos: Vec<f64>) -> Particle {
        let l = initial_pos.len();
        Particle {
            p_best: initial_pos.clone(),
            pos: initial_pos,
            velocity: vec![0f64; l] 
        }
        
    }
}

