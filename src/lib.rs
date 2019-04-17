pub mod client;
pub mod protocol;
pub mod server;
mod util;

extern crate proc_macro;
extern crate rayon;
extern crate uuid;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
