use super::pso::{pareto_front, dominates, Particle, Swarm};
use rand::random;
use std::fs::write;

// #[test]
// fn test_update_velocity(){
//     let len = 2;
//     let mut swarm = Swarm::new (
//         2,
//         vec![
//         (-100f64,100f64),
//         (-100f64,100f64)
//         ],
//         1.2f64,
//         1.2f64,
//         2f64);
//     println!("{:?}", swarm);
//     swarm.update_velocity();
//     println!("{:?}", swarm);
// }

fn points_to_string(points: &[Vec<f64>]) -> String {
    points.iter()
        .map(|xs| format!("{}\t{}", xs[0], xs[1]))
        .collect::<Vec<String>>().join("\n")
}

#[test]
fn test_pareto_front() {
    // testing pareto front on sine
    let f_x: fn(f64) -> f64 = |x| x.sin() / x * 100.0;

    let len = 10000;
    let mut points: Vec<Vec<f64>> = Vec::with_capacity(len);
    let interval = (0f64, 30f64);
    let sampling_rate: f64 = (interval.1 - interval.0) / len as f64; 

    for i in 0..len {
        let x = interval.0 + i as f64 * sampling_rate;
        points.push(vec![x, f_x(x)]);
    }

    write("domination_base.dat", points_to_string(&points)).unwrap();

    let pareto_front = pareto_front(&points, &[true, false]);
    let pareto_front: Vec<Vec<f64>> = pareto_front.iter()
        .map(|i| points[*i].clone()).collect();

    write("domination_front.dat", points_to_string(&pareto_front)).unwrap();
}

// #[test]
// fn text_random_float(){
//     for _ in 0..100 {
//         println!("{}", rand::random::<f64>());
//     }
// }
