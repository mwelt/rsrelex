
struct Swarm {
    learning_cognitive: f64,
    learning_social: f64,   
    inertia: f64,
    best: Vec<f64>,
    particles: Vec<Particle>
}

impl Swarm {
    fn update_velocity(&mut self){

        for particle in self.particles.iter_mut() {
            for (i, x_i) in particle.pos.iter_mut().enumerate() {
                let c1r1 = 
                    self.learning_cognitive * rand::random::<f64>();
                let c2r2 = 
                    self.learning_social * rand::random::<f64>();

                let v_i = self.inertia * particle.velocity[i]
                    + c1r1 * (particle.p_best[i] - *x_i)  
                    + c2r2 * (self.best[i] - *x_i);

                *x_i += v_i;
            }
        }
    }
}

struct Particle {
    p_best: Vec<f64>,
    pos: Vec<f64>,
    velocity: Vec<f64> 
}

impl Particle {
    fn new(initial_pos: Vec<f64>) -> Particle {
        let l = initial_pos.len();
        Particle {
            p_best: initial_pos.clone(),
            pos: initial_pos,
            velocity: Vec::with_capacity(l).iter()
                .map(|_:&f64| 0f64).collect()
        }
        
    }
}

