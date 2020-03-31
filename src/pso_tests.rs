use super::pso::*;
// use rand::random;
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

#[test]
fn test_swarm_init() {
    let fitness_fn: FitnessFn = |pos| vec![pos[0]/10.0, pos[1]/10.0];

    let position_bounds: Vec<Bound> = vec![
        (-100.0, 100.0),
        (-100.0, 100.0)
        ];

    let fitness_bounds: Vec<Bound> = vec![ 
        (-10.0, 10.0),
        (-10.0, 10.00)
        ];

    // model a simple fitness landscape:
    // field of 200x200 [-100,100]
    // optimization of [x/10, y/10] with 
    // x to max and y to min -> should yield  
    // optimal pareto of [0,0]
    let mut swarm = Swarm::new(
        100,
        1.0,
        1.0,
        2.0,
        position_bounds,
        fitness_bounds,
        vec![true, false],
        fitness_fn
    );

    // first, check if all initialized particles
    // are in the range of fitness landscape
    for particle in swarm.particles.iter() {
        for (i, (l, h)) in swarm.position_bounds.iter().enumerate() {
            assert_eq!(true, particle.position[i] >= *l);
            assert_eq!(true, particle.position[i] <= *h);
        }
    }

    // double check pareto front 
    let particle_fitnesss: Vec<&Fitness> = swarm.particles.iter()
        .map(|p| &p.fitness).collect();

    let pareto_fitnesss: Vec<&Fitness> = pareto_front(
        &particle_fitnesss,
        &swarm.fitness_pareto_directions).iter()
        .map(|i| particle_fitnesss[*i])
        .collect();

    for f in pareto_fitnesss.iter() {
        let mut is_dominated = false;
        for f_ in pareto_fitnesss.iter() {
            if dominates(f_, f, &swarm.fitness_pareto_directions){
                is_dominated = true;
                break;
            }
        }
        assert_eq!(false, is_dominated);
    }

    // check if leader fitness is really pareto optimal
    for leader in swarm.leaders.iter() {
        let mut is_dominated = false;
        for particle in swarm.particles.iter() {
            if dominates(&particle.fitness,
                &leader.fitness, &swarm.fitness_pareto_directions) {
                is_dominated = true;
                break;
            }
        }
        
        assert_eq!(false, is_dominated);
    }

    let particle_positions: Vec<&Position> = swarm.particles.iter()
        .map(|p| &p.position).collect();
    write("swarm_init_particles.dat", 
        points_to_string_(&particle_positions)).unwrap();

    let leaders_positions: Vec<&Position> = swarm.leaders.iter()
        .map(|l| &l.position).collect();

    write("swarm_init_leaders.dat", 
        points_to_string_(&leaders_positions)).unwrap();

    let particle_fitnesss: Vec<&Fitness> = swarm.particles.iter()
        .map(|p| &p.fitness).collect();
    write("swarm_init_particle_fitnesss.dat", 
        points_to_string_(&particle_fitnesss)).unwrap();

    let leader_fitnesss: Vec<&Fitness> = swarm.leaders.iter()
        .map(|l| &l.fitness).collect();
    write("swarm_init_leader_fitnesss.dat",
        points_to_string_(&leader_fitnesss)).unwrap();
    
}

fn points_to_string_(points: &[&Vec<f64>]) -> String {
    points.iter()
        .map(|xs| format!("{}\t{}", xs[0], xs[1]))
        .collect::<Vec<String>>().join("\n")
}

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

    let points: Vec<&Vec<f64>> = points.iter().map(|p| p).collect();

    let pareto_front = pareto_front(&points, &[true, false]);
    let pareto_front: Vec<Vec<f64>> = pareto_front.iter()
        .map(|i| points[*i].clone()).collect();

    // just double check pareto front
    for p in pareto_front.iter() {
        let mut is_dominated = false;
        for p_ in points.iter() {
            if dominates(&p_, &p, &[true, false]) {
                is_dominated = true;
                break;
            }
        }
        assert_eq!(false, is_dominated);
    }

    write("domination_front.dat", points_to_string(&pareto_front)).unwrap();
}

// #[test]
// fn text_random_float(){
//     for _ in 0..100 {
//         println!("{}", rand::random::<f64>());
//     }
// }
