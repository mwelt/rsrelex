use super::*;
use atty::Stream;
use std::io::{self, BufRead};

#[test]
fn test_read_words_from_stdin(){
    if atty::is(Stream::Stdin) {
        println!("Not pipe");
    }
    else{
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            println!("{}", line.expect("error reading line from stdin")); 
        }
    }
}
