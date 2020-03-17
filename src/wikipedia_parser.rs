use regex::Regex;

pub fn parse(s: &str) -> bool {

    lazy_static! {
        static ref RE_CURLY_BRACES: Regex = 
            Regex::new(r"(\{\{.*\}\})").unwrap();
    } 

    RE_CURLY_BRACES.is_match(s)
}
