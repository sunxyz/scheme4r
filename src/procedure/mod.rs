mod apply;
mod branch;
mod define;
mod eval;
mod lambda;
mod numbers;
mod quote;
mod set;
mod syntax;
mod types;
use crate::{
    env::{Env, EnvOps},
    types::{
        func::ApplyArgs,
        refs::RefOps,
        Number,
        Type::{self, *},
        *,
    },
};

pub fn init_procedures(env: &mut Env) {
    numbers::reg_procedures(env);
    define::reg_procedures(env);
    lambda::reg_procedures(env);
    set::reg_procedures(env);
    quote::reg_procedures(env);
    branch::reg_procedure(env);
    syntax::reg_procedure(env);
    apply::reg_procedure(env);
    eval::reg_procedure(env);
    types::reg_procedure(env);
}

impl Env {
    fn reg_procedure(&mut self, name: &str, proc: fn(&mut ApplyArgs) -> Type) {
        self.define(name, Type::procedure_of(name, proc));
    }
}
