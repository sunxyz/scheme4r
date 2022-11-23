use crate::{env::Env, types::{Type, List}};

pub fn reg_procedures(env: &mut Env) {
    env.reg_procedure("quote", |args| {
        let t = args.expr().car();
        if let Type::Lists(_) = t{
            let v = parser_quote(&t);
            return v;
        }else{
            t
        }
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
            Type::Lists( List::of_quote(vec))
        }
        _ => Type::Lists(List::of_quote(vec![Type::symbol("quote"),t.clone()])),
    }
}
