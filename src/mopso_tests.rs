use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use ndarray::stack;
use std::io::Write;
use std::fs::{File, OpenOptions};

fn append_swarm_to_file(s: &mopso::Swarm, f: &str){

   let mut file = OpenOptions::new()
       .create(true)
       .append(true)
       .open(f)
       .expect(&format!("Unable to open {}.", f));

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

#[test]
fn test_mopso(){
   let mut swarm = mopso::Swarm::new(
        50,
        2,
        array![
        [-100.0, 100.0],
        [-100.0, 100.0],
        ],
        2,
        array![
        [-10.0, 10.0], 
        [-10.0, 10.0]
        ],
        array![true, true],
        2,
        &|swarm: &mut mopso::Swarm| {
            let f_idx = swarm.o.f;
            let o = &swarm.o;
            swarm.particles.axis_iter_mut(Axis(0))
                .into_par_iter()
                .for_each(|mut p| {
                    let pos = p.slice(s![o.p.0..o.p.1]); 
                    let p0 = pos[0];
                    let p1 = pos[1];
                    p[o.f.0] = p0;
                    p[o.f.0 + 1] = p1;
                    // p[o.f.0] = p0 / 10.0;
                    // p[o.f.0 + 1] = p1 / 10.0;
                    // p[f_idx] = 4.0 * pos[0].sin().powi(2) + 4.0 * pos[1].sin().powi(2)
                });
        },
        &|i: usize, swarm: &mut mopso::Swarm| {
            append_swarm_to_file(swarm, "test_pso_swarm.dat");
            println!("{:?}", swarm.leaders_pos_by_rank);
        });

   swarm.fly(
       100, 
       &pso::HyperParams {
           learning_cognitive: 0.1,
           learning_social: 0.1,
           inertia: 0.02
       },
       &|swarm: &mut mopso::Swarm| {
           let f_idx = swarm.o.f;
           let o = &swarm.o;
           swarm.particles.axis_iter_mut(Axis(0))
               .into_par_iter()
               .for_each(|mut p| {
                   let pos = p.slice(s![o.p.0..o.p.1]); 
                   let p0 = pos[0];
                   let p1 = pos[1];
                   p[o.f.0] = p0 / 10.0;
                   p[o.f.0 + 1] = p1 / 10.0;
                   // p[f_idx] = 4.0 * pos[0].sin().powi(2) + 4.0 * pos[1].sin().powi(2)
               });
       },
       &|i: usize, swarm: &mut mopso::Swarm| {
           append_swarm_to_file(swarm, "test_pso_swarm.dat");
       });
}


#[test]
fn test_matrix_stacking(){
    let m1 = array![[1, 2, 3],[4, 5, 6]];
    let m2 = array![[7, 8, 9],[10, 11, 12]];

    let m3 = stack![Axis(0), m1, m2];

    // let m4 = m3.axis_iter(Axis(0)).filter(|row|
    //     row[0] == 1).collect::<ArrayBase<f64, Ix1>>();

    // println!("{:?}", m4);
}

#[test]
fn test_pareto(){
    assert!(mopso::dominates(
            &array![90.0,90.0].view(), 
            &array![80.0,80.0].view(), 
            &array![true, true].view()));




}

// use super::mopso::*;
// use std::fs::write;
// // use assert_approx_eq::assert_approx_eq;

// struct TestFitnessFn {
// }

// impl FitnessFn for TestFitnessFn {
//     fn calc_fitness(&self, pos: &Position) -> Fitness {
//         vec![pos[0]/10.0, pos[1]/10.0]
//     }
// }

// fn init_swarm<'a>(fitness_fn: &'a TestFitnessFn) -> Swarm<'a, TestFitnessFn> {
//     // let fitness_fn: FitnessFn = |pos| vec![pos[0].sin(), pos[1].cos()];

//     let position_bounds: Vec<Bound> = vec![
//         (-100.0, 100.0),
//         (-100.0, 100.0)
//         ];

//     let fitness_bounds: Vec<Bound> = vec![ 
//         (-10.0, 10.0),
//         (-10.0, 10.00)
//         ];

//     // model a simple fitness landscape:
//     // field of 200x200 [-100,100]
//     // optimization of [x/10, y/10] with 
//     // x to max and y to min -> should yield  
//     // optimal pareto of [0,0]
//     Swarm::<TestFitnessFn>::new(
//         50,
//         0.1,
//         0.1,
//         0.02,
//         position_bounds,
//         fitness_bounds,
//         vec![true, false],
//         fitness_fn
//     )
// }

// fn test_leader_pareto(swarm: &Swarm<TestFitnessFn>){
    
//     // check if leader fitness is really pareto optimal
//     for leader in swarm.leaders.iter() {
//         let mut is_dominated = false;
//         for particle in swarm.particles.iter() {
//             if dominates(&particle.fitness,
//                 &leader.fitness, &swarm.fitness_pareto_directions) {
//                 is_dominated = true;
//                 break;
//             }
//         }
        
//         assert_eq!(false, is_dominated);
//     }
// }

// fn test_particles_in_bounds(swarm: &Swarm<TestFitnessFn>) {
//     for particle in swarm.particles.iter() {
//         for (i, x_i) in particle.position.iter().enumerate() {
//             assert!(*x_i >= swarm.position_bounds[i].0);
//             assert!(*x_i <= swarm.position_bounds[i].1);
//         }
//     }
// }

// fn write_swarm_dat(swarm: &Swarm<TestFitnessFn>, file_name_prefix: &str, 
//     file_name_suffix: &str, write_fitness: bool) {
    
//     // write updated movement data
//     let particle_positions: Vec<&Position> = swarm.particles.iter()
//         .map(|p| &p.position).collect();
//     write([file_name_prefix, "sp_", file_name_suffix, ".dat"].join(""), 
//         points_to_string_(&particle_positions)).unwrap();

//     let leaders_positions: Vec<&Position> = swarm.leaders.iter()
//         .map(|l| &l.position).collect();

//     write([file_name_prefix, "spl_", file_name_suffix, ".dat"].join(""), 
//         points_to_string_(&leaders_positions)).unwrap();

//     if write_fitness {
//         // write updated fitness data
//         let particle_fitnesss: Vec<&Fitness> = swarm.particles.iter()
//             .map(|p| &p.fitness).collect();
//         write([file_name_prefix, "sf_", file_name_suffix, ".dat"].join(""), 
//             points_to_string_(&particle_fitnesss)).unwrap();

//         let leader_fitnesss: Vec<&Fitness> = swarm.leaders.iter()
//             .map(|l| &l.fitness).collect();
//         write([file_name_prefix, "sfl_", file_name_suffix, ".dat"].join(""),
//         points_to_string_(&leader_fitnesss)).unwrap();
//     }
// }

// fn on_iteration(i: usize, swarm: &Swarm<TestFitnessFn>){
//     // check if all leader are pareto for each step
//     test_leader_pareto(swarm);
//     test_particles_in_bounds(swarm);
//     write_swarm_dat(swarm, "fly_test/", &(i+1).to_string(), false);
// }

// #[test]
// fn mopso_test_swarm_fly() {
//     let mut swarm = init_swarm(&TestFitnessFn{});
//     write_swarm_dat(&swarm, "fly_test/", "0", false);
//     swarm.fly(100, &on_iteration);
// }

// #[test]
// fn mopso_test_swarm_init() {

//     let swarm = init_swarm(&TestFitnessFn{});
    
//     // first, check if all initialized particles
//     // are in the range of fitness landscape
//     for particle in swarm.particles.iter() {
//         for (i, (l, h)) in swarm.position_bounds.iter().enumerate() {
//             assert_eq!(true, particle.position[i] >= *l);
//             assert_eq!(true, particle.position[i] <= *h);
//         }
//     }

//     test_leader_pareto(&swarm);

//     for l in swarm.leaders.iter() {
//         println!("({}, {:?}, {})", l.rank, l.fitness, l.crowding_distance);
//     }
//     println!("rank sum: {}", swarm.rank_sum);
//     // write_swarm_dat(&swarm, "", "init", true); 
    
// }

// fn points_to_string_(points: &[&Vec<f64>]) -> String {
//     points.iter()
//         .map(|xs| format!("{}\t{}", xs[0], xs[1]))
//         .collect::<Vec<String>>().join("\n")
// }

// fn points_to_string(points: &[Vec<f64>]) -> String {
//     points.iter()
//         .map(|xs| format!("{}\t{}", xs[0], xs[1]))
//         .collect::<Vec<String>>().join("\n")
// }

// #[test]
// fn mopso_test_pareto_front() {
//     // testing pareto front on sine
//     let f_x: fn(f64) -> f64 = |x| x.sin() / x * 100.0;

//     let len = 10000;
//     let mut points: Vec<Vec<f64>> = Vec::with_capacity(len);
//     let interval = (0f64, 30f64);
//     let sampling_rate: f64 = (interval.1 - interval.0) / len as f64; 

//     for i in 0..len {
//         let x = interval.0 + i as f64 * sampling_rate;
//         points.push(vec![x, f_x(x)]);
//     }

//     write("domination_base.dat", points_to_string(&points)).unwrap();

//     let points: Vec<&Vec<f64>> = points.iter().map(|p| p).collect();

//     let pareto_front = pareto_front(&points, &[true, false]);
//     let pareto_front: Vec<Vec<f64>> = pareto_front.iter()
//         .map(|i| points[*i].clone()).collect();

//     // just double check pareto front
//     for p in pareto_front.iter() {
//         let mut is_dominated = false;
//         for p_ in points.iter() {
//             if dominates(&p_, &p, &[true, false]) {
//                 is_dominated = true;
//                 break;
//             }
//         }
//         assert_eq!(false, is_dominated);
//     }

//     write("domination_front.dat", points_to_string(&pareto_front)).unwrap();
// }

// #[test]
// fn mopso_test_domination_zero_zero(){
//     assert_eq!(false, dominates(&[0.0, 0.0], &[0.0, 0.0], &[true, true]));
// }

// #[test]
// fn mopso_test_pareto_front_zero_zero(){
//     assert_eq!(0, pareto_front(
//         &[
//         &vec![0f64, 0f64],
//         &vec![0f64, 0f64],
//         &vec![0f64, 0f64]
//         ], &[true, true]).len());
// }
// // #[test]
// // fn test_distance(){
// //     assert_approx_eq!(0f64, distance(&[0.0, 0.0], &[0.0, 0.0]));
// //     assert_approx_eq!(2f64.sqrt(), distance(&[0.0, 0.0], &[1.0, 1.0]));
// //     assert_approx_eq!(2f64.sqrt(), distance(&[0.0, 0.0], &[-1.0, 1.0]));
// // }

