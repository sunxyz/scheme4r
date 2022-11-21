

use super::*;
use crate::env::{Env};

pub fn reg_procedures(env: &mut Env) {
    env.reg_procedure("define", |args| {
        let expr = args.expr();
        if let Symbols(key) =  expr.car(){
            let v = args.clone().inter(&Lists(expr.cdr()));
            args.env().ref_write().define(key.as_str(), v);
            Nil
        } else {
            panic!("define: invalid argument");
        } 
    })
}
