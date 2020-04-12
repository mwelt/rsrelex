use assert_approx_eq::assert_approx_eq;
use super::mopso_train::{calc_precision_recall, read_word_file};
use super::bootstrap;

#[test]
fn test_precision_recall() {
    let reference = [1, 2, 3, 4, 5, 6, 7, 8, 9, 0];
    let retrival_erg = [2, 4, 7, 8, 10, 23, 123];

    let (precision, recall) = calc_precision_recall(&retrival_erg, &reference); 
    assert_approx_eq!(4.0 / 7.0, precision);
    assert_approx_eq!(4.0 / 10.0, recall);
}

#[test]
fn test_read_reference() {
    let reference = "countries.txt";
    let env = bootstrap("test_bin".to_string()); 

    let word_nrs = read_word_file(&reference, &env);
    word_nrs.iter().for_each(|w_nr| println!("{}", w_nr));
}
