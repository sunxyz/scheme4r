

use super::*;

pub fn reg_procedure(env: &mut Env) {
    env.reg_procedure("list",|args| Type::Lists(args.args().clone()));
}
