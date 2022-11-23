mod define;
mod lambda;
mod numbers;
mod set;
mod quote;
mod branch;
 mod syntax;
 mod apply;
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
    syntax::reg_procedure(env);
    apply::reg_procedure(env);
}

impl Env {
    fn reg_procedure(&mut self, name: &str, proc: fn(&mut ApplyArgs) -> Type) {
        self. define(name, Type::procedure_of(name, proc));
    }
    fn reg_procedure0(&mut self, proc: Type){
        if let Procedures(p) = proc.clone(){
            self. define(p.get_name(), proc);
        }
    }
}

