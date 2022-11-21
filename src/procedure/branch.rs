use super::*;
use crate::utils::bool_utils::is_true;

fn if0(apply_args: &mut ApplyArgs) -> Type {
    let list = apply_args.expr();
    if list.len() > 1 {
        let cond = list.car();
        let then = list.cdr().car();
        let else_ = if list.len() > 2 {
            Lists(list.cdr().cdr())
        } else {
            Nil
        };
        if is_true(&apply_args.inter(&cond)) {
            apply_args.inter(&then)
        } else {
            if let Lists(else__) = else_ {
                apply_args.inter(&Lists(else__.clone()))
            } else {
                Nil
            }
        }
    } else {
        panic!("if: wrong number of arguments");
    }
}

pub fn reg_procedure(env: &mut Env) {
    env.reg_procedure("if", if0);
}
