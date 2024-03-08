#![allow(unused_imports)]
#![allow(dead_code)]
pub mod data_structures;
pub mod env;
pub mod error_code;
pub mod log;
pub mod utils;

#[macro_use]
extern crate lazy_static;
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
