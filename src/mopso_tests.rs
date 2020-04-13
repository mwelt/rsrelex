use super::*;
use ndarray::prelude::*;
use ndarray::parallel::prelude::*;
use ndarray::stack;
use std::io::Write;
use std::fs::OpenOptions;

fn append_swarm_to_file(s: &mopso::Swarm, f: &str){

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
            let o = &swarm.o;
            swarm.particles.axis_iter_mut(Axis(0))
                .into_par_iter()
                .for_each(|mut p| {
                    let pos = p.slice(s![o.p.0..o.p.1]); 
                    let p0 = pos[0];
                    let p1 = pos[1];
                    p[o.f.0] = p0;
                    p[o.f.0 + 1] = p1;
                });
        },
        &|_i: usize, swarm: &mut mopso::Swarm| {
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
           let o = &swarm.o;
           swarm.particles.axis_iter_mut(Axis(0))
               .into_par_iter()
               .for_each(|mut p| {
                   let pos = p.slice(s![o.p.0..o.p.1]); 
                   let p0 = pos[0];
                   let p1 = pos[1];
                   p[o.f.0] = p0 / 10.0;
                   p[o.f.0 + 1] = p1 / 10.0;
               });
       },
       &|_i: usize, swarm: &mut mopso::Swarm| {
           append_swarm_to_file(swarm, "test_pso_swarm.dat");
       });
}


#[test]
fn test_matrix_stacking(){
    let m1 = array![[1, 2, 3],[4, 5, 6]];
    let m2 = array![[7, 8, 9],[10, 11, 12]];

    let _m3 = stack![Axis(0), m1, m2];

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
