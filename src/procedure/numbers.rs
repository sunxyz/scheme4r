use super::*;

fn add(apply_args: &mut ApplyArgs) -> Type {
    calc(apply_args, |a, b| a + b)
}

fn subtract(apply_args: &mut ApplyArgs) -> Type {
    calc(apply_args, |a, b| a - b)
}

fn multiply(apply_args: &mut ApplyArgs) -> Type {
    calc(apply_args, |a, b| a * b)
}

fn divide(apply_args: &mut ApplyArgs) -> Type {
    calc(apply_args, |a, b| a / b)
}

fn calc(apply_args: &mut ApplyArgs, f: fn(Number, Number) -> Number) -> Type {
    Type::Numbers(
        apply_args
            .args()
            .data()
            .iter()
            .map(|x| -> Number {
                match x {
                    Type::Numbers(n) => *n,
                    v => panic!(" not a number {}", v),
                }
            })
            .reduce(f)
            .unwrap(),
    )
}

pub fn reg_procedures(env: &mut Env) {
    env.reg_procedure("+", add);
    env.reg_procedure("-", subtract);
    env.reg_procedure("*", multiply);
    env.reg_procedure("/", divide);
}
