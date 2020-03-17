use super::*;
use std::fs::read_to_string;

#[test]
fn test_wp_parser(){
    let txt = read_to_string("wp.txt")
        .expect("Could not read test file wp.txt."); 
    assert_eq!(wikipedia_parser::parse(&txt), true);
}
