pub mod parser;
mod interpreter;
mod env;
mod types;
mod procedure;
mod utils;
#[cfg(test)]
mod tests;

pub use interpreter::eval;
