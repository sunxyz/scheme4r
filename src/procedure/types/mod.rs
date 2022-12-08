use super::*;
mod numbers;
mod  list;

pub fn reg_procedure(env: &mut Env) {
    numbers::reg_procedures(env);
    list::reg_procedure(env);
}
