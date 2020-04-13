use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use std::io::Write;
use std::fs::OpenOptions;

fn append_swarm_to_file(s: &pso::Swarm, f: &str){

   let mut file = OpenOptions::new()
       .create(true)
       .append(true)
       .open(f)
       .unwrap_or_else(|_| panic!("Unable to open {}.", f));

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
        &|_i: usize, swarm: &mut pso::Swarm| {
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
       &|_i: usize, swarm: &mut pso::Swarm| {
           append_swarm_to_file(swarm, "test_pso_swarm.dat");
       });

}

#[test]
fn test_matrix(){

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
    }];

    println!("{:?}", m.slice(s![.., 0..3]));

}
