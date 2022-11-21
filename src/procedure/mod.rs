mod define;
mod lambda;
mod numbers;
mod set;
mod quote;
mod branch;
use crate::{
    env::{Env, EnvOps},
    types::{
        func::ApplyArgs,
        refs::RefOps,
        Number,
        Type::{self, *},
        *
    },
};

pub fn init_procedures(env: &mut Env) {
    numbers::reg_procedures(env);
    define::reg_procedures(env);
    lambda::reg_procedures(env);
    set::reg_procedures(env);
    quote::reg_procedures(env);
    branch::reg_procedure(env);
}

impl Env {
    fn reg_procedure(&mut self, name: &str, proc: fn(&mut ApplyArgs) -> Type) {
        self. define(name, Type::procedure_of(name, proc));
    }
}

