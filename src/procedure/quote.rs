use crate::{env::Env, types::Type};

pub fn reg_procedures(env: &mut Env) {
    env.reg_procedure("quote", |args| {
        let t = args.expr().car();
        let v = parser_quote(&t);
        args.inter(&v)
    })
}

fn parser_quote(t: &Type) -> Type {
    match t {
        Type::Lists(l) => {
            let mut vec = vec![Type::symbol("list")];
            vec.extend(
                l.data()
                    .iter()
                    .map(|x| parser_quote(x))
                    .collect::<Vec<Type>>(),
            );
            Type::expr_of(vec)
        }
        _ => Type::quote(t.clone()),
    }
}
