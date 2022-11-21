use super::*;
use crate::env::{Env, EnvOps};

pub fn reg_procedures(env: &mut Env) {
    env.reg_procedure("set!", |args| {
        let expr = args.expr();
        if let Symbols(key) = expr.car() {
            let v = args.clone().inter(&Lists(expr.cdr()));
            args.env().ref_write().set(key.as_str(), v);
            Nil
        } else {
            panic!("set!: invalid argument");
        }
    })
}
