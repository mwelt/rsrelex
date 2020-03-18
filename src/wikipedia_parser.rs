use regex::Regex;

pub fn parse(s: &str) -> String {

    lazy_static! {
        static ref RE_CURLY_BRACES: Regex = 
            Regex::new(r"(\{\{(?P<c>[^\}]*)\}\})").unwrap();
        static ref RE_LINK_BRACES: Regex =
            Regex::new(r"(\[\[(?P<c>File:[^\]|\|]*).*\]\])").unwrap();
        static ref RE_WP_LINK_BRACES: Regex =
            Regex::new(r"(\[\[(?P<c>[^\]|\|]*).*\]\])").unwrap();
        static ref RE_HEADLINE: Regex =
            Regex::new(r"(==(?P<c>[^=]*)==)").unwrap();
    } 

    let s = RE_CURLY_BRACES.replace_all(s, "");
    let s = RE_LINK_BRACES.replace_all(&*s, "");
    let s = RE_HEADLINE.replace_all(&*s, "$c");
    RE_WP_LINK_BRACES.replace_all(&*s, "$c").into()

}
