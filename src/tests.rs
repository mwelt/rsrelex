use super::*;
use std::fs::{read_to_string, write};

#[test]
fn test_wp_parser(){
    assert_eq!(wikipedia_parser::parse("{{abc}}efg{{hij}}"), "efg");
    assert_eq!(wikipedia_parser::parse("ab[[cd|ef]]hi"), "abcdhi");
    assert_eq!(wikipedia_parser::parse("ab[[cd|ef]]hi[[jkl]]mn[[o|pq]]rs"), "abcdhijklmnors");
    assert_eq!(wikipedia_parser::parse("== abcd =="), " abcd ");
}

// #[test]
fn test_wp_parser_on_file(){
    let txt = read_to_string("wp.txt")
        .expect("Could not read test file wp.txt."); 
    let txt_ = wikipedia_parser::parse(&txt);
    write("wp_.txt", txt_);
}
