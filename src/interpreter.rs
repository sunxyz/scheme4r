

use crate::{
    env::{Env, EnvOps, RefEnv},
    parser::parser,
    procedure::init_procedures,
    types::{
        func::ApplyArgs,
        refs::RefOps,
        Type::{self, *},
        *,
    },
};

pub fn eval(exp: &str) -> Result<Type, &str> {
    let root = Env::root();
    // let v = root.write();
    init_procedures(&mut root.borrow_mut());
    let env = Env::extend(root);
    let exp = parser(exp.to_string()).expect("parser error");
    // println!("exp:{}", exp);
    Ok(interpreter0(&exp, env))
}

pub fn interpreter(exp: &SExpr, env: RefEnv) -> Type {
    let car = exp.car();
    // println!("car: {}", car);
    let cdr = &exp.cdr();
    // println!("cdr: {} is-exp:{}", cdr, cdr.is_expr());
    match car {
        Symbols(key) => {
            let v = env
                .ref_read()
                .get(key.as_str())
                .expect(format!("Undefined variable: {}", key).as_str());
            if let Procedures(f) = v.clone() {
                if exp.is_expr() {
                    // println!("exc: {}", cdr);
                    return apply(f, cdr, env);
                }
            }
            if cdr.is_nil() {
                v
            } else {
                interpreter(cdr, env)
            }
        }
        Lists(l) => {
            let v = interpreter(&l, env.clone());
            if let Procedures(f) = v.clone() {
                if exp.is_expr() {
                    // println!("exc0: {}", cdr);
                    return apply(f, cdr, env);
                }
            }
            if cdr.is_nil() {
                v
            } else {
                interpreter(cdr, env)
            }
        }
        _ => {
            if cdr.is_nil() {
                car
            } else {
                interpreter(cdr, env)
            }
        }
    }
}

fn apply(
    f: Procedure, //Func,//fn(&mut ApplyArgs) -> LispType,
    cdr: &SExpr,
    env: RefEnv,
) -> Type {
    let lazy_args_f: fn(List, RefEnv) -> List = |exp, e| {
        let mut t = List::new();
        for l in exp.data() {
            t.push(interpreter0(&l, e.clone()));
        }
        t
    };

    f.call(&mut ApplyArgs::new(
        cdr.clone(),
        None,
        lazy_args_f,
        interpreter0,
        env,
    ))
}

fn interpreter0(o: &Type, env: RefEnv) -> Type {
    match o {
        Lists(l) => interpreter(l, env),
        Symbols(s) => env
            .ref_read()
            .get(s.as_str())
            .expect(format!("undefined symbol {}", s.as_str()).as_str()),
        _ => o.clone(),
    }
}
