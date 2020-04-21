use super::*;

#[test]
fn test_conex_config() {
    let config_file = "conex.toml";

    let config: ConexConfig = 
        toml::from_str(&read_to_string(config_file)
            .unwrap_or_else(|_| panic!("Unable to open file \"{}\".", config_file)))
        .unwrap();
}


// type Closure = dyn Fn(&Vec<usize>) -> Vec<usize>;

// struct ClosureHolder<'a> {
//     closure_fn: &'a Closure
// }

// impl ClosureHolder<'_> {
//     fn new<'a>(closure_fn: &'a Closure) -> ClosureHolder<'a> {
//         ClosureHolder {
//             closure_fn
//         }
//     }
// }

// fn do_smthg(f: &Closure) -> Vec<usize> {
//     f(&vec![1, 2, 3, 4, 5])
// }

// fn do_smthg_w_closure_holder(closure_holder: &ClosureHolder) -> Vec<usize> {
//     (closure_holder.closure_fn)(&vec![1, 2, 3, 4, 5])
// }

// #[test]
// fn test_closure_borrow(){
//     let x = 3;
//     let f = |xs: &Vec<usize>| xs.iter().map(|e| e + 3).collect();

//     let vs: &Vec<usize> = &vec![1, 2, 3, 4, 5];
//     let f_ = |xs: &Vec<usize>| vs.clone();

//     let closure_holder = ClosureHolder::new(&f);
//     let closure_holder_ = ClosureHolder::new(&f_);

//     println!("{:?}", do_smthg(&f));
//     println!("{:?}", do_smthg(&f_));

//     println!("{:?}", do_smthg_w_closure_holder(&closure_holder));
// }
