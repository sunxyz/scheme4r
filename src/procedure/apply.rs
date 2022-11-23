use super::*;

pub fn reg_procedure(env: &mut Env) {
    env.reg_procedure("apply",|args| args.apply());
}
