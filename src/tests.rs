use super::*;
use std::fs::{read_to_string, write};

#[test]
fn test_multiline_lists(){

    let big_test_str = "*{{cite journal|last=Antliff|first=Mark |year=1998|title=Cubism, Futurism, Anarchism: The 'Aestheticism' of the Action d'art Group, 1906-1920|journal=Oxford Art Journal|volume=21|issue=2|pages=101–120 |ref=harv|jstor=1360616 |doi=10.1093/oxartj/21.2.99 }}
* {{cite book |last=Avrich |first=Paul|title=Anarchist Voices: An Oral History of Anarchism in America |year=1996|publisher=Princeton University Press |isbn=978-0-691-04494-1|ref=harv}}
* {{cite book |last=Bantman|first=Constance|chapter=The Era of Propaganda by the Deed|pages=371–388|editor1-last=Levy|editor1-first=Carl|editor1-link=Carl Levy (political scientist)|editor2-last=Adams|editor2-first=Matthew S. |title=The Palgrave Handbook of Anarchism|year=2019|publisher=[[Springer Publishing]]|isbn=978-3-319-75620-2|ref=harv}}";

    assert_eq!(wikipedia_parser::strip_markup(big_test_str), ""); 

    // ;
    assert_eq!(wikipedia_parser::strip_markup("abcd; efg\n"), "abcd; efg\n");
    assert_eq!(wikipedia_parser::strip_markup("\n; abcdefg\n"), "\n");
    assert_eq!(wikipedia_parser::strip_markup("\n  ; abcdefg\n"), "\n");

    // lists
    assert_eq!(wikipedia_parser::strip_markup("\n     ** this is an item\n"), "\n");
    assert_eq!(wikipedia_parser::strip_markup("\n     ; this is an item\n"), "\n");
    assert_eq!(wikipedia_parser::strip_markup("\n** this is an item\n"), "\n");
    assert_eq!(wikipedia_parser::strip_markup("\n## this is an item\n"), "\n");

}

#[test]
fn test_links(){

    // internal wikilinks
    assert_eq!(wikipedia_parser::strip_markup("ab[[cd|ef]]hi"), "abcdhi");
    assert_eq!(wikipedia_parser::strip_markup("ab[[cd|ef]]hi[[jkl]]mn[[o|pq]]rs"), "abcdhijklmnors");

    // file links
    assert_eq!(wikipedia_parser::strip_markup("ab[[File:cd|ef]]hi"), "abhi");
    
}

#[test]
fn test_wiki_markup(){
    
    // every wikimarkup {{..}}
    assert_eq!(wikipedia_parser::strip_markup("{{abc}}efg{{hij}}"), "efg");

    // every wikimarkup {|..|}
    assert_eq!(wikipedia_parser::strip_markup("{|abc|ijk|}efg{|hij|}"), "efg");

    // headlines
    assert_eq!(wikipedia_parser::strip_markup("= abcd ="), "");
    assert_eq!(wikipedia_parser::strip_markup("=== abcd ==="), "");
    assert_eq!(wikipedia_parser::strip_markup("==== abcd ===="), "");
    assert_eq!(wikipedia_parser::strip_markup("===== abcd ====="), "");

    // horizontal rules
    assert_eq!(wikipedia_parser::strip_markup("--------\n"), "\n");

}

#[test]
fn test_xhtml_markup(){
    
    // comments
    assert_eq!(wikipedia_parser::strip_markup("&lt;!--abc--&gt;def<!--hij-->"), "def");

    // xhtml fragments
    assert_eq!(wikipedia_parser::strip_markup("&quot;abc&quot;"), "abc");

    // xhtml tags
    assert_eq!(wikipedia_parser::strip_markup("<foo>abcde fghi</foo>"), "");
    assert_eq!(wikipedia_parser::strip_markup("<small font-size=\"12\">abcde fghi</small>"), "abcde fghi");
    assert_eq!(wikipedia_parser::strip_markup("<big font-size=\"12\">abcde fghi</big>"), "abcde fghi");
    assert_eq!(wikipedia_parser::strip_markup("<blockquote font-size=\"12\">abcde fghi</blockquote>"), "abcde fghi");
    assert_eq!(wikipedia_parser::strip_markup("<poem font-size=\"12\">abcde fghi</poem>"), "abcde fghi");
    assert_eq!(wikipedia_parser::strip_markup("<big>foo <small font-size=\"12\">bar</small> baz</big>"), "foo bar baz");

    assert_eq!(wikipedia_parser::strip_markup("<nowiki/>"), "");
    assert_eq!(wikipedia_parser::strip_markup("<nowiki />"), "");

}
    

// #[test]
fn test_wp_parser_on_file(){
    let txt = read_to_string("wp.txt")
        .expect("Could not read test file wp.txt."); 
    let txt_ = wikipedia_parser::strip_markup(&txt);
    write("wp_.txt", txt_).expect("could not write wp_.txt");
}
