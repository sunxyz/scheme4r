use crate::{
    error::SchemeError,
    reader::{Datum, Reader},
    runtime::{procedure::Procedure, EnvRef, Environment, Value},
};

pub struct Engine {
    root_env: EnvRef,
}

impl Engine {
    pub fn new(root_env: EnvRef) -> Self {
        Self { root_env }
    }

    pub fn run(&self, source: &str) -> Result<Value, SchemeError> {
        let forms = Reader::new(source).read_all()?;
        let mut result = Value::Unspecified;
        for form in forms {
            result = self.eval(&form, self.root_env.clone())?;
        }
        Ok(result)
    }

    fn eval(&self, expr: &Datum, env: EnvRef) -> Result<Value, SchemeError> {
        match expr {
            Datum::Boolean(value) => Ok(Value::Boolean(*value)),
            Datum::Number(value) => Ok(Value::Number(*value)),
            Datum::String(value) => Ok(Value::String(value.clone())),
            Datum::Symbol(name) => env.borrow().lookup(name),
            Datum::EmptyList => Ok(Value::EmptyList),
            Datum::Pair(_, _) => self.eval_list_form(expr, env),
        }
    }

    fn eval_list_form(&self, expr: &Datum, env: EnvRef) -> Result<Value, SchemeError> {
        let items = expr.collect_proper_list().ok_or_else(|| {
            SchemeError::syntax("expected a proper list in application position", None)
        })?;

        let Some(first) = items.first() else {
            return Err(SchemeError::syntax("cannot evaluate an empty list", None));
        };

        if let Some(symbol) = first.as_symbol() {
            match symbol {
                "quote" => return self.eval_quote(&items),
                "if" => return self.eval_if(&items, env),
                "define" => return self.eval_define(&items, env),
                "lambda" => return self.eval_lambda(&items, env, None),
                "begin" => return self.eval_begin(&items[1..], env),
                "set!" => return self.eval_set(&items, env),
                _ => {}
            }
        }

        let operator = self.eval(first, env.clone())?;
        let mut args = Vec::new();
        for expr in &items[1..] {
            args.push(self.eval(expr, env.clone())?);
        }
        self.apply(operator, args)
    }

    fn eval_quote(&self, items: &[&Datum]) -> Result<Value, SchemeError> {
        if items.len() != 2 {
            return Err(SchemeError::arity("'quote' expects exactly 1 argument"));
        }
        Ok(datum_to_value(items[1]))
    }

    fn eval_if(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 || items.len() > 4 {
            return Err(SchemeError::arity(
                "'if' expects 2 or 3 arguments after the keyword",
            ));
        }

        let predicate = self.eval(items[1], env.clone())?;
        if predicate.is_truthy() {
            self.eval(items[2], env)
        } else if items.len() == 4 {
            self.eval(items[3], env)
        } else {
            Ok(Value::Unspecified)
        }
    }

    fn eval_define(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'define' expects a name and at least one body expression",
            ));
        }

        match items[1] {
            Datum::Symbol(name) => {
                if items.len() != 3 {
                    return Err(SchemeError::syntax(
                        "variable define must contain exactly one value expression",
                        None,
                    ));
                }
                let value = self.eval(items[2], env.clone())?;
                env.borrow_mut().define(name.clone(), value);
                Ok(Value::Unspecified)
            }
            Datum::Pair(_, _) => {
                let signature = items[1].collect_proper_list().ok_or_else(|| {
                    SchemeError::syntax("function signature must be a proper list", None)
                })?;

                let Some(Datum::Symbol(name)) = signature.first() else {
                    return Err(SchemeError::syntax("function name must be a symbol", None));
                };

                let params = extract_parameters(&signature[1..])?;
                let body = items[2..]
                    .iter()
                    .map(|datum| (*datum).clone())
                    .collect::<Vec<_>>();
                let proc = Value::lambda(Some(name.clone()), params, body, env.clone());
                env.borrow_mut().define(name.clone(), proc);
                Ok(Value::Unspecified)
            }
            _ => Err(SchemeError::syntax(
                "define expects a symbol or function signature",
                None,
            )),
        }
    }

    fn eval_lambda(
        &self,
        items: &[&Datum],
        env: EnvRef,
        name: Option<String>,
    ) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'lambda' expects a parameter list and at least one body expression",
            ));
        }
        let params = items[1]
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("lambda parameter list must be proper", None))?;
        let params = extract_parameters(&params)?;
        let body = items[2..]
            .iter()
            .map(|datum| (*datum).clone())
            .collect::<Vec<_>>();
        Ok(Value::lambda(name, params, body, env))
    }

    fn eval_begin(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        let mut result = Value::Unspecified;
        for expr in items {
            result = self.eval(expr, env.clone())?;
        }
        Ok(result)
    }

    fn eval_set(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() != 3 {
            return Err(SchemeError::arity("'set!' expects exactly 2 arguments"));
        }
        let Datum::Symbol(name) = items[1] else {
            return Err(SchemeError::syntax("'set!' target must be a symbol", None));
        };
        let value = self.eval(items[2], env.clone())?;
        env.borrow_mut().set(name, value)?;
        Ok(Value::Unspecified)
    }

    fn apply(&self, proc: Value, args: Vec<Value>) -> Result<Value, SchemeError> {
        match proc {
            Value::Procedure(proc_ref) => match proc_ref.as_ref() {
                Procedure::Builtin { func, .. } => func(&args),
                Procedure::Lambda {
                    params, body, env, ..
                } => {
                    if params.len() != args.len() {
                        return Err(SchemeError::arity(format!(
                            "lambda expected {} arguments, got {}",
                            params.len(),
                            args.len()
                        )));
                    }

                    let call_env = Environment::child(env.clone());
                    {
                        let mut call_env_mut = call_env.borrow_mut();
                        for (name, value) in params.iter().zip(args.into_iter()) {
                            call_env_mut.define(name.clone(), value);
                        }
                    }

                    let body_refs = body.iter().collect::<Vec<_>>();
                    self.eval_begin(&body_refs, call_env)
                }
            },
            other => Err(SchemeError::type_error(format!(
                "attempted to call a non-procedure: {other}"
            ))),
        }
    }
}

fn datum_to_value(datum: &Datum) -> Value {
    match datum {
        Datum::Boolean(value) => Value::Boolean(*value),
        Datum::Number(value) => Value::Number(*value),
        Datum::String(value) => Value::String(value.clone()),
        Datum::Symbol(value) => Value::symbol(value.clone()),
        Datum::EmptyList => Value::EmptyList,
        Datum::Pair(car, cdr) => Value::pair(datum_to_value(car), datum_to_value(cdr)),
    }
}

fn extract_parameters(items: &[&Datum]) -> Result<Vec<String>, SchemeError> {
    let mut params = Vec::new();
    for item in items {
        match item {
            Datum::Symbol(name) => params.push(name.clone()),
            _ => {
                return Err(SchemeError::syntax(
                    "parameter list must contain only symbols",
                    None,
                ))
            }
        }
    }
    Ok(params)
}
