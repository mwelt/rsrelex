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

        ps.column_mut(o.pbf).iter_mut()
            .for_each(|f| *f = fitness_bounds.0);

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
                let row_ = row.slice(s![o.p.0 ..= o.p.1]).into_owned();

                //TODO: see mopso copy position + fitness to personal best 
                row.slice_mut(s![o.pb.0 ..= o.pb.1])
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

        (1..=n).for_each(|i| {
            self.update_position(hp);
            f(self);
            self.set_personal_bests();
            self.find_and_set_leader();
            c(i, self);
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
