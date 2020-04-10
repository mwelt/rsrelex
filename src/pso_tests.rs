use super::pso::*;
use std::fs::write;
// use assert_approx_eq::assert_approx_eq;

struct TestFitnessFn {
}

impl FitnessFn for TestFitnessFn {
    fn calc_fitness(&self, pos: &Position) -> Fitness {
        4.0 * pos[0].sin().powi(2) + 4.0 * pos[1].sin().powi(2)
    }
}

fn init_swarm<'a>(fitness_fn: &'a TestFitnessFn) -> Swarm<'a, TestFitnessFn> {
    // let fitness_fn: FitnessFn = |pos| vec![pos[0].sin(), pos[1].cos()];

    let position_bounds: Vec<Bound> = vec![
        (0.0, 6.0),
        (0.0, 6.0)
        ];

    Swarm::<TestFitnessFn>::new(
        50,
        0.1,
        0.1,
        0.02,
        position_bounds,
        (0.0, 10.0),
        fitness_fn
    )
}

fn points_to_string_(points: &[&Vec<f64>]) -> String {
    points.iter()
        .map(|xs| format!("{}\t{}", xs[0], xs[1]))
        .collect::<Vec<String>>().join("\n")
}

fn write_swarm_dat(swarm: &Swarm<TestFitnessFn>, file_name_prefix: &str, 
    file_name_suffix: &str, write_fitness: bool) {
    
    // write updated movement data
    let particle_positions: Vec<&Position> = swarm.particles.iter()
        .map(|p| &p.position).collect();
    write([file_name_prefix, "sp_", file_name_suffix, ".dat"].join(""), 
        points_to_string_(&particle_positions)).unwrap();
}

#[test]
fn test_swarm_fly_pso(){
    let mut swarm = init_swarm(&TestFitnessFn{});
    write_swarm_dat(&swarm, "pso_test/", &"0", true);
    swarm.fly(100, &|i, swarm| 
        write_swarm_dat(&swarm, "pso_test/", &(i+1).to_string(), true));
}
