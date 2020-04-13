use super::pso::{HyperParams, Bounds, Particles};
use ndarray::prelude::*;
use ndarray::stack;
use log::info;
use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

#[derive(Debug)]
pub struct Offsets {
    pub p: (usize, usize),
    pub v: (usize, usize),
    pub pb: (usize, usize),
    pub l: (usize, usize),
    pub f: (usize, usize),
    pub pbf: (usize, usize)
}

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
   
    #[allow(clippy::too_many_arguments)]
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

pub fn dominates(
    x: &ArrayView1<f64>, 
    y: &ArrayView1<f64>, 
    directions: &ArrayView1<bool>) -> bool {

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
