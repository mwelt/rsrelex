use regex::Regex;

pub fn strip_markup(s: &str) -> String {

    lazy_static! {
        static ref REMOVE: Vec<&'static str> =  vec!(
            "&quot;", 
            "'''''", 
            "'''", 
            "''",
        );

        static ref REPLACE: Vec<(&'static str, &'static str)> = vec!(
            ("&gt;", ">"),
            ("&lt;", "<")
        );

        static ref REGEX: Vec<(Regex, &'static str)> = vec!(
            // lists, bullet points 
            (Regex::new(r"(?m)(^\s*[*#:;]+[^\n]*$)").unwrap(), ""),

            // every wikimarkup {{...}}
            (Regex::new(r"(\{\{(?P<c>[^}]*)\}\})").unwrap(), ""),
            // every wikimarkup {|...|}
            (Regex::new(r"(\{\|(?P<c>[^}]*)\|\})").unwrap(), ""),

            // every File, Media or Category link 
            (Regex::new(r"(\[\[(?P<c>:?(File|Category|Media):[^]]*).*\]\])").unwrap(), ""),

            // modify wiki internal links
            (Regex::new(r"(\[\[#?(?P<c>[^]|]*)\]\])").unwrap(), "$c"),
            (Regex::new(r"(\[\[#?[^|]*\|(?P<c>[^]]*)\]\])").unwrap(), "$c"),

            // headlines
            (Regex::new(r"(===== (?P<c>[^=]+) =====)").unwrap(), ""),
            (Regex::new(r"(==== (?P<c>[^=]+) ====)").unwrap(), ""),
            (Regex::new(r"(=== (?P<c>[^=]+) ===)").unwrap(), ""),
            (Regex::new(r"(== (?P<c>[^=]+) ==)").unwrap(), ""),
            (Regex::new(r"(= (?P<c>[^=]+) =)").unwrap(), ""),

            // comments
            (Regex::new(r"(<!--(?P<c>[^-]*)-->)").unwrap(), ""),
            // horizontal rules
            (Regex::new(r"(?m)(^\-+$)").unwrap(), ""),

            //inline html
            (Regex::new(r"<small[^>]*>([^<]*)</small>").unwrap(), "$1"),
            (Regex::new(r"<big[^>]*>([^<]*)</big>").unwrap(), "$1"),
            (Regex::new(r"<blockquote[^>]*>([^<]*)</blockquote>").unwrap(), "$1"),
            (Regex::new(r"<poem[^>]*>([^<]*)</poem>").unwrap(), "$1"),
            (Regex::new(r"<[^>]*>[^<]*</[^>]*>").unwrap(), ""),
            (Regex::new(r"<[^/]*/>").unwrap(), "")

        );
    } 

    let mut s: String = s.into(); 

    for (search, replace) in REPLACE.iter() {
        s = s.replace(search, replace);
    }
     
    for rem in REMOVE.iter() {
        s = s.replace(rem, "");
    }

    for (regex, replace) in REGEX.iter() {
        s = regex.replace_all(&s, replace as &str).into();
    }
    s
}
