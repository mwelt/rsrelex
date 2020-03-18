use regex::Regex;

pub fn strip_markup(s: &str) -> String {

    lazy_static! {
        static ref REMOVE: Vec<String> =  vec!(
            "&quot;".into(), "'''".into(), "''".into() 
        );

        static ref REGEX: Vec<(Regex, String)> = vec!(
            (Regex::new(r"(\n)(\s*(;|[*#:]+)[^\n]*\n)").unwrap(), "$1".into()),

            // // every wikimarkup {{...}}
            // (Regex::new(r"(\{\{(?P<c>[^}]*)\}\})").unwrap(), "".into()),
            // // every wikimarkup {|...|}
            // (Regex::new(r"(\{\|(?P<c>[^}]*)\|\})").unwrap(), "".into()),
            // // every File link 
            // (Regex::new(r"(\[\[(?P<c>File:[^]]*).*\]\])").unwrap(), "".into()),
            // // every Category link
            // (Regex::new(r"(\[\[(?P<c>Category:[^]]*).*\]\])").unwrap(), "".into()),
            // // modify wiki internal links
            // (Regex::new(r"(\[\[(?P<c>[^]|]*)\|?[^]]*\]\])").unwrap(), "$c".into()),
            // // headlines
            // (Regex::new(r"(===== (?P<c>[^=]+) =====)").unwrap(), "".into()),
            // (Regex::new(r"(==== (?P<c>[^=]+) ====)").unwrap(), "".into()),
            // (Regex::new(r"(=== (?P<c>[^=]+) ===)").unwrap(), "".into()),
            // (Regex::new(r"(== (?P<c>[^=]+) ==)").unwrap(), "".into()),
            // (Regex::new(r"(= (?P<c>[^=]+) =)").unwrap(), "".into()),

            // // comments
            // (Regex::new(r"(&lt;!--(?P<c>[^-]*)--&gt;)").unwrap(), "".into()),
            // // horizontal rules
            // (Regex::new(r"(\-+\n)").unwrap(), "".into()),
            
            // (Regex::new(r"(\n)(\s*;[^\n]*\n)").unwrap(), "$1".into()),
            // (Regex::new(r"(\n)(\s*[*#:]+[^\n]*\n)").unwrap(), "$1".into()),
            
            // (Regex::new(r"[\n|\^](\v*[*#;:]+ [^\n]*\n)").unwrap(), "".into())
        );
    } 

    let mut s: String = s.into(); 
    for (regex, replace) in REGEX.iter() {
        s = regex.replace_all(&s, replace as &str).into();
    }
    for rem in REMOVE.iter() {
        s = s.replace(rem, "");
    }
    s
}
