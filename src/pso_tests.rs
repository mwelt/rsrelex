use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use std::io::Write;
use std::fs::{File, OpenOptions};

fn append_swarm_to_file(s: &pso::Swarm, f: &str){

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

    //write new line to seperate iterations 
    writeln!(file, "\n").unwrap();
}

#[test]
fn test_pso(){

   let mut swarm = pso::Swarm::new(
        50,
        2,
        array![
        [0.0, 6.0],
        [0.0, 6.0],
        ],
        (0.0, 10.0),
        2,
        &|swarm: &mut pso::Swarm| {
            let f_idx = swarm.o.f;
            let o = &swarm.o;
            swarm.particles.axis_iter_mut(Axis(0))
                .into_par_iter()
                .for_each(|mut p| {
                    let pos = p.slice(s![o.p.0..o.p.1]); 
                    p[f_idx] = 4.0 * pos[0].sin().powi(2) + 4.0 * pos[1].sin().powi(2)
                });
        },
        &|i: usize, swarm: &mut pso::Swarm| {
            append_swarm_to_file(swarm, "test_pso_swarm.dat");
        });

   swarm.fly(
       100, 
       &pso::HyperParams {
           learning_cognitive: 0.1,
           learning_social: 0.1,
           inertia: 0.02
       },
       &|swarm: &mut pso::Swarm| {
           let f_idx = swarm.o.f;
           let o = &swarm.o;
           swarm.particles.axis_iter_mut(Axis(0))
               .into_par_iter()
               .for_each(|mut p| {
                   let pos = p.slice(s![o.p.0..o.p.1]); 
                   p[f_idx] = 4.0 * pos[0].sin().powi(2) + 4.0 * pos[1].sin().powi(2)
               });
       },
       &|i: usize, swarm: &mut pso::Swarm| {
           append_swarm_to_file(swarm, "test_pso_swarm.dat");
       });
   

   // swarm.particles.slice_mut(
   //     s![.., swarm.offsets[[pso::VELOCITY, 0]]..swarm.offsets[[pso::VELOCITY, 1]]])
   //     .iter_mut().for_each(|x| *x = 22.0);

}

// #[test]
fn test_matrix(){

    let rng = rand::thread_rng();
    
    let mut m: Array<f64, Ix2> = Array::zeros((10, 6));

    let dim = 3;

    let p: Array<f64, Ix2> = array![
        [-100.0, 100.0],
        [-20.0, 20.0],
        [-5.0, 5.0]];

    println!("{:?}", p);

    azip![(
        mut a_col in m.slice_mut(s![.., 0..dim]).gencolumns_mut(), 
        a_bound_row in p.genrows()) {

        a_col.iter_mut().for_each(|x| *x = a_bound_row[0]);
           // for x in a_col.iter_mut(){
           //     *x = a_bound_row[0];
           // }
    }];

    println!("{:?}", m.slice(s![.., 0..3]));

}

// use std::fs::write;
// // use assert_approx_eq::assert_approx_eq;

// struct TestFitnessFn {
// }

// impl FitnessFn for TestFitnessFn {
//     fn calc_fitness(&self, pos: &Position) -> Fitness {
//         4.0 * pos[0].sin().powi(2) + 4.0 * pos[1].sin().powi(2)
//     }
// }

// fn init_swarm<'a>(fitness_fn: &'a TestFitnessFn) -> Swarm<'a, TestFitnessFn> {
//     // let fitness_fn: FitnessFn = |pos| vec![pos[0].sin(), pos[1].cos()];

//     let position_bounds: Vec<Bound> = vec![
//         (0.0, 6.0),
//         (0.0, 6.0)
//         ];

//     Swarm::<TestFitnessFn>::new(
//         50,
//         0.1,
//         0.1,
//         0.02,
//         position_bounds,
//         (0.0, 10.0),
//         fitness_fn
//     )
// }

// fn points_to_string_(points: &[&Vec<f64>]) -> String {
//     points.iter()
//         .map(|xs| format!("{}\t{}", xs[0], xs[1]))
//         .collect::<Vec<String>>().join("\n")
// }

// fn write_swarm_dat(swarm: &Swarm<TestFitnessFn>, file_name_prefix: &str, 
//     file_name_suffix: &str, write_fitness: bool) {
    
//     // write updated movement data
//     let particle_positions: Vec<&Position> = swarm.particles.iter()
//         .map(|p| &p.position).collect();
//     write([file_name_prefix, "sp_", file_name_suffix, ".dat"].join(""), 
//         points_to_string_(&particle_positions)).unwrap();
// }

// #[test]
// fn test_swarm_fly_pso(){
//     let mut swarm = init_swarm(&TestFitnessFn{});
//     write_swarm_dat(&swarm, "pso_test/", &"0", true);
//     swarm.fly(100, &|i, swarm| 
//         write_swarm_dat(&swarm, "pso_test/", &(i+1).to_string(), true));
// }
