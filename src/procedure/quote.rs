use crate::env::Env;

pub fn reg_procedures(env: &mut Env) {
    env.reg_procedure("quote", |args| args.expr().car())
}
