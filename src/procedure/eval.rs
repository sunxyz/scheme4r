use super::*;

pub fn reg_procedure(env: &mut Env) {
    env.reg_procedure("eval",|args| {
        let v = &Type::Lists(args.args().clone());
        args.inter(v)
    });
}
