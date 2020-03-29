use super::pso::{Particle, Swarm};
use rand::random;

#[test]
fn test_update_velocity(){
    let len = 2;
    let mut swarm = Swarm::new (
        2,
        vec![
        (-100f64,100f64),
        (-100f64,100f64)
        ],
        1.2f64,
        1.2f64,
        2f64);
    println!("{:?}", swarm);
    swarm.update_velocity();
    println!("{:?}", swarm);
}
