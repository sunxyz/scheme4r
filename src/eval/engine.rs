use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    error::SchemeError,
    eval::syntax::SyntaxRules,
    reader::{Datum, Reader},
    runtime::{
        procedure::Procedure, EnvRef, Environment, Library, Port, PortRef, RecordFieldSpec,
        RecordInstance, RecordType, Value,
    },
};

#[derive(Clone)]
struct PortState {
    current_input: PortRef,
    current_output: PortRef,
    current_error: PortRef,
}

pub struct Engine {
    root_env: EnvRef,
    port_state: Rc<RefCell<PortState>>,
    env_stack: RefCell<Vec<EnvRef>>,
}

impl Engine {
    pub fn new(root_env: EnvRef) -> Self {
        Self {
            root_env,
            port_state: Rc::new(RefCell::new(PortState {
                current_input: Port::stdin(),
                current_output: Port::stdout(),
                current_error: Port::stdout(),
            })),
            env_stack: RefCell::new(Vec::new()),
        }
    }

    pub fn run(&self, source: &str) -> Result<Value, SchemeError> {
        self.run_in_env(source, self.root_env.clone())
    }

    pub(crate) fn run_in_env(&self, source: &str, env: EnvRef) -> Result<Value, SchemeError> {
        let forms = Reader::new(source).read_all()?;
        let mut result = Value::Unspecified;
        for form in forms {
            result = self.eval_datum(&form, env.clone())?;
        }
        Ok(result)
    }

    pub(crate) fn eval_datum(&self, expr: &Datum, env: EnvRef) -> Result<Value, SchemeError> {
        self.eval(expr, env)
    }

    pub(crate) fn current_env(&self) -> EnvRef {
        self.env_stack
            .borrow()
            .last()
            .cloned()
            .unwrap_or_else(|| self.root_env.clone())
    }

    pub(crate) fn current_input_port(&self) -> PortRef {
        self.port_state.borrow().current_input.clone()
    }

    pub(crate) fn current_output_port(&self) -> PortRef {
        self.port_state.borrow().current_output.clone()
    }

    pub(crate) fn current_error_port(&self) -> PortRef {
        self.port_state.borrow().current_error.clone()
    }

    pub(crate) fn require_single_value(
        &self,
        value: Value,
        context: &str,
    ) -> Result<Value, SchemeError> {
        match value {
            Value::Multiple(values) => {
                let mut iter = values.into_iter();
                match (iter.next(), iter.next()) {
                    (Some(value), None) => Ok(value),
                    _ => Err(SchemeError::runtime(format!(
                        "{context} expected a single value"
                    ))),
                }
            }
            other => Ok(other),
        }
    }

    fn eval_single(&self, expr: &Datum, env: EnvRef, context: &str) -> Result<Value, SchemeError> {
        let value = self.eval(expr, env)?;
        self.require_single_value(value, context)
    }

    fn eval(&self, expr: &Datum, env: EnvRef) -> Result<Value, SchemeError> {
        match expr {
            Datum::Boolean(value) => Ok(Value::Boolean(*value)),
            Datum::Number(value) => Ok(Value::Number(*value)),
            Datum::Character(value) => Ok(Value::Character(*value)),
            Datum::String(value) => Ok(Value::string(value.clone())),
            Datum::Vector(values) => Ok(Value::vector(values.iter().map(datum_to_value).collect())),
            Datum::ByteVector(values) => Ok(Value::bytevector(values.clone())),
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
                "quasiquote" => return self.eval_quasiquote(&items, env),
                "if" => return self.eval_if(&items, env),
                "define" => return self.eval_define(&items, env),
                "define-record-type" => return self.eval_define_record_type(&items, env),
                "define-syntax" => return self.eval_define_syntax(&items, env),
                "import" => return self.eval_import(&items[1..], env),
                "define-library" => return self.eval_define_library(&items, env),
                "lambda" => return self.eval_lambda(&items, env, None),
                "begin" => return self.eval_begin(&items[1..], env),
                "set!" => return self.eval_set(&items, env),
                "and" => return self.eval_and(&items[1..], env),
                "or" => return self.eval_or(&items[1..], env),
                "let" => return self.eval_let(&items, env),
                "let*" => return self.eval_let_star(&items, env),
                "letrec" => return self.eval_letrec(&items, env),
                "let-syntax" => return self.eval_let_syntax(&items, env),
                "letrec-syntax" => return self.eval_letrec_syntax(&items, env),
                "cond" => return self.eval_cond(&items[1..], env),
                "case" => return self.eval_case(&items[1..], env),
                "do" => return self.eval_do(&items, env),
                _ => {}
            }

            let transformer = { env.borrow().lookup_syntax(symbol) };
            if let Some(transformer) = transformer {
                let expanded = transformer.expand(expr)?;
                return self.eval(&expanded, env);
            }
        }

        let operator = self.eval_single(first, env.clone(), "procedure position")?;
        let mut args = Vec::new();
        for expr in &items[1..] {
            args.push(self.eval_single(expr, env.clone(), "argument position")?);
        }
        self.apply(operator, env, args)
    }

    fn eval_quote(&self, items: &[&Datum]) -> Result<Value, SchemeError> {
        if items.len() != 2 {
            return Err(SchemeError::arity("'quote' expects exactly 1 argument"));
        }
        Ok(datum_to_value(items[1]))
    }

    fn eval_quasiquote(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() != 2 {
            return Err(SchemeError::arity(
                "'quasiquote' expects exactly 1 argument",
            ));
        }
        self.quasiquote(items[1], env, 1)
    }

    fn eval_if(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 || items.len() > 4 {
            return Err(SchemeError::arity(
                "'if' expects 2 or 3 arguments after the keyword",
            ));
        }

        let predicate = self.eval_single(items[1], env.clone(), "'if' predicate")?;
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
                let value = self.eval_single(items[2], env.clone(), "'define' value")?;
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
                let proc = Value::lambda(Some(name.clone()), params, None, body, env.clone());
                env.borrow_mut().define(name.clone(), proc);
                Ok(Value::Unspecified)
            }
            _ => Err(SchemeError::syntax(
                "define expects a symbol or function signature",
                None,
            )),
        }
    }

    fn eval_define_record_type(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 4 {
            return Err(SchemeError::arity(
                "'define-record-type' expects a type name, constructor, predicate, and field specs",
            ));
        }

        let Datum::Symbol(type_name) = items[1] else {
            return Err(SchemeError::syntax(
                "'define-record-type' type name must be a symbol",
                None,
            ));
        };

        let (constructor_name, constructor_fields) = parse_record_constructor_spec(items[2])?;

        let Datum::Symbol(predicate_name) = items[3] else {
            return Err(SchemeError::syntax(
                "'define-record-type' predicate name must be a symbol",
                None,
            ));
        };

        let field_specs = parse_record_field_specs(&items[4..])?;

        if constructor_fields.len() != field_specs.len() {
            return Err(SchemeError::syntax(
                "'define-record-type' constructor field list must match field spec count",
                None,
            ));
        }

        for (index, field_name) in constructor_fields.iter().enumerate() {
            if field_specs[index].field_name != *field_name {
                return Err(SchemeError::syntax(
                    "'define-record-type' constructor field order must match field specs",
                    None,
                ));
            }
        }

        let record_type = RecordType::new(
            type_name.clone(),
            field_specs
                .iter()
                .map(|spec| RecordFieldSpec::new(spec.field_name.clone(), spec.mutator.is_some()))
                .collect(),
        );

        let mut env_mut = env.borrow_mut();

        env_mut.define(
            constructor_name.clone(),
            Value::Procedure(Procedure::record_constructor(
                constructor_name.clone(),
                record_type.clone(),
            )),
        );
        env_mut.define(
            predicate_name.clone(),
            Value::Procedure(Procedure::record_predicate(
                predicate_name.clone(),
                record_type.clone(),
            )),
        );

        for (index, spec) in field_specs.iter().enumerate() {
            env_mut.define(
                spec.accessor.clone(),
                Value::Procedure(Procedure::record_accessor(
                    spec.accessor.clone(),
                    record_type.clone(),
                    index,
                )),
            );

            if let Some(mutator) = &spec.mutator {
                env_mut.define(
                    mutator.clone(),
                    Value::Procedure(Procedure::record_mutator(
                        mutator.clone(),
                        record_type.clone(),
                        index,
                    )),
                );
            }
        }

        Ok(Value::Unspecified)
    }

    fn eval_internal_define(&self, datum: &Datum, env: EnvRef) -> Result<(), SchemeError> {
        let items = datum
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("'define' form must be proper", None))?;

        match items[1] {
            Datum::Symbol(name) => {
                if items.len() != 3 {
                    return Err(SchemeError::syntax(
                        "variable define must contain exactly one value expression",
                        None,
                    ));
                }
                let value = self.eval_single(items[2], env.clone(), "'define' value")?;
                env.borrow_mut().set(name, value)?;
                Ok(())
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
                let proc = Value::lambda(Some(name.clone()), params, None, body, env.clone());
                env.borrow_mut().set(name, proc)?;
                Ok(())
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
        let (params, rest) = extract_formals(items[1])?;
        let body = items[2..]
            .iter()
            .map(|datum| (*datum).clone())
            .collect::<Vec<_>>();
        Ok(Value::lambda(name, params, rest, body, env))
    }

    fn eval_define_syntax(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() != 3 {
            return Err(SchemeError::arity(
                "'define-syntax' expects a name and a transformer specification",
            ));
        }

        let Datum::Symbol(name) = items[1] else {
            return Err(SchemeError::syntax(
                "'define-syntax' requires a symbol name",
                None,
            ));
        };

        let transformer = SyntaxRules::compile(items[2])?;
        env.borrow_mut().define_syntax(name.clone(), transformer);
        Ok(Value::Unspecified)
    }

    fn eval_import(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.is_empty() {
            return Err(SchemeError::arity(
                "'import' expects at least one import set",
            ));
        }

        let mut imported = HashMap::new();
        for item in items {
            imported.extend(resolve_import_set(item, env.clone())?);
        }
        env.borrow_mut().import_bindings(&imported);
        Ok(Value::Unspecified)
    }

    fn eval_define_library(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'define-library' expects a name and declarations",
            ));
        }

        let library_name = parse_library_name(items[1])?;
        let library_env = Environment::isolated_with_registry(env.clone());
        let mut exports = Vec::new();
        let mut begin_forms = Vec::new();

        for declaration in &items[2..] {
            let parts = declaration.collect_proper_list().ok_or_else(|| {
                SchemeError::syntax("'define-library' declarations must be proper lists", None)
            })?;
            let Some(keyword) = parts.first().and_then(|item| item.as_symbol()) else {
                return Err(SchemeError::syntax(
                    "'define-library' declarations require a keyword",
                    None,
                ));
            };

            match keyword {
                "export" => exports.extend(parse_export_specs(&parts[1..])?),
                "import" => {
                    let mut imported = HashMap::new();
                    for import_set in &parts[1..] {
                        imported.extend(resolve_import_set(import_set, env.clone())?);
                    }
                    library_env.borrow_mut().import_bindings(&imported);
                }
                "begin" => begin_forms.extend(parts[1..].iter().copied()),
                _ => {
                    return Err(SchemeError::syntax(
                        format!("unsupported library declaration: {keyword}"),
                        None,
                    ));
                }
            }
        }

        if !begin_forms.is_empty() {
            self.eval_begin(&begin_forms, library_env.clone())?;
        }

        let exported_bindings = collect_library_exports(&library_env, &exports)?;
        env.borrow_mut()
            .define_library(library_name, Library::new(exported_bindings));
        Ok(Value::Unspecified)
    }

    fn eval_let_syntax(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        self.eval_local_syntax(items, env, false)
    }

    fn eval_letrec_syntax(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        self.eval_local_syntax(items, env, true)
    }

    fn eval_local_syntax(
        &self,
        items: &[&Datum],
        env: EnvRef,
        _recursive: bool,
    ) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "local syntax forms expect bindings and at least one body expression",
            ));
        }

        let syntax_env = Environment::child(env);
        let bindings = parse_syntax_bindings(items[1])?;
        {
            let mut syntax_env_mut = syntax_env.borrow_mut();
            for (name, transformer) in bindings {
                syntax_env_mut.define_syntax(name, transformer);
            }
        }
        self.eval_body(&items[2..], syntax_env)
    }

    fn eval_begin(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        let mut result = Value::Unspecified;
        for expr in items {
            result = self.eval(expr, env.clone())?;
        }
        Ok(result)
    }

    fn eval_body(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        let define_count = items
            .iter()
            .take_while(|item| is_definition_form(item))
            .count();

        if define_count == 0 {
            return self.eval_begin(items, env);
        }

        let body_env = Environment::child(env);
        {
            let mut body_env_mut = body_env.borrow_mut();
            for define in &items[..define_count] {
                body_env_mut.define(extract_define_name(define)?, Value::Unspecified);
            }
        }

        for define in &items[..define_count] {
            self.eval_internal_define(define, body_env.clone())?;
        }

        self.eval_begin(&items[define_count..], body_env)
    }

    fn eval_and(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        let mut result = Value::Boolean(true);
        for expr in items {
            result = self.eval_single(expr, env.clone(), "'and' expression")?;
            if !result.is_truthy() {
                return Ok(result);
            }
        }
        Ok(result)
    }

    fn eval_or(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        for expr in items {
            let value = self.eval_single(expr, env.clone(), "'or' expression")?;
            if value.is_truthy() {
                return Ok(value);
            }
        }
        Ok(Value::Boolean(false))
    }

    fn eval_let(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'let' expects bindings and at least one body expression",
            ));
        }

        match items[1] {
            Datum::Symbol(name) => {
                if items.len() < 4 {
                    return Err(SchemeError::arity(
                        "named 'let' expects bindings and at least one body expression",
                    ));
                }
                let bindings = parse_bindings(items[2])?;
                let mut params = Vec::new();
                let mut args = Vec::new();
                for (param, init) in bindings {
                    params.push(param);
                    args.push(self.eval_single(init, env.clone(), "'let' binding init")?);
                }

                let let_env = Environment::child(env);
                let body = items[3..]
                    .iter()
                    .map(|datum| (*datum).clone())
                    .collect::<Vec<_>>();
                let proc = Value::lambda(Some(name.clone()), params, None, body, let_env.clone());
                let_env.borrow_mut().define(name.clone(), proc.clone());
                self.apply(proc, let_env, args)
            }
            _ => {
                let bindings = parse_bindings(items[1])?;
                let let_env = Environment::child(env.clone());
                {
                    let mut let_env_mut = let_env.borrow_mut();
                    for (name, init) in bindings {
                        let value = self.eval_single(init, env.clone(), "'let' binding init")?;
                        let_env_mut.define(name, value);
                    }
                }
                self.eval_body(&items[2..], let_env)
            }
        }
    }

    fn eval_let_star(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'let*' expects bindings and at least one body expression",
            ));
        }

        let bindings = parse_bindings(items[1])?;
        let let_env = Environment::child(env);
        for (name, init) in bindings {
            let value = self.eval_single(init, let_env.clone(), "'let*' binding init")?;
            let_env.borrow_mut().define(name, value);
        }
        self.eval_body(&items[2..], let_env)
    }

    fn eval_letrec(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'letrec' expects bindings and at least one body expression",
            ));
        }

        let bindings = parse_bindings(items[1])?;
        let let_env = Environment::child(env);
        {
            let mut let_env_mut = let_env.borrow_mut();
            for (name, _) in &bindings {
                let_env_mut.define(name.clone(), Value::Unspecified);
            }
        }
        for (name, init) in bindings {
            let value = self.eval_single(init, let_env.clone(), "'letrec' binding init")?;
            let_env.borrow_mut().set(&name, value)?;
        }
        self.eval_body(&items[2..], let_env)
    }

    fn eval_cond(&self, clauses: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        for (index, clause) in clauses.iter().enumerate() {
            let entries = clause
                .collect_proper_list()
                .ok_or_else(|| SchemeError::syntax("'cond' clauses must be proper lists", None))?;

            let Some(test) = entries.first() else {
                return Err(SchemeError::syntax("'cond' clauses cannot be empty", None));
            };

            if test.as_symbol() == Some("else") {
                if index + 1 != clauses.len() {
                    return Err(SchemeError::syntax("'cond' else clause must be last", None));
                }
                if entries.len() == 1 {
                    return Err(SchemeError::syntax(
                        "'cond' else clause must contain at least one expression",
                        None,
                    ));
                }
                return self.eval_begin(&entries[1..], env.clone());
            }

            let test_value = self.eval_single(test, env.clone(), "'cond' test expression")?;
            if !test_value.is_truthy() {
                continue;
            }

            if entries.len() == 1 {
                return Ok(test_value);
            }

            if entries.get(1).and_then(|datum| datum.as_symbol()) == Some("=>") {
                if entries.len() != 3 {
                    return Err(SchemeError::syntax(
                        "'cond' => clause must contain exactly one recipient expression",
                        None,
                    ));
                }
                let recipient = self.eval_single(entries[2], env.clone(), "'cond' recipient")?;
                return self.apply(recipient, env.clone(), vec![test_value]);
            }

            return self.eval_begin(&entries[1..], env.clone());
        }

        Ok(Value::Unspecified)
    }

    fn eval_case(&self, clauses: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        let (key_expr, clause_exprs) = clauses
            .split_first()
            .ok_or_else(|| SchemeError::arity("'case' expects a key and at least one clause"))?;
        let key = self.eval_single(key_expr, env.clone(), "'case' key expression")?;

        for (index, clause) in clause_exprs.iter().enumerate() {
            let entries = clause
                .collect_proper_list()
                .ok_or_else(|| SchemeError::syntax("'case' clauses must be proper lists", None))?;

            let Some(head) = entries.first() else {
                return Err(SchemeError::syntax("'case' clauses cannot be empty", None));
            };

            if head.as_symbol() == Some("else") {
                if index + 1 != clause_exprs.len() {
                    return Err(SchemeError::syntax("'case' else clause must be last", None));
                }
                if entries.len() == 1 {
                    return Err(SchemeError::syntax(
                        "'case' else clause must contain at least one expression",
                        None,
                    ));
                }
                return self.eval_begin(&entries[1..], env.clone());
            }

            let datums = head.collect_proper_list().ok_or_else(|| {
                SchemeError::syntax("'case' clause datum list must be proper", None)
            })?;

            let matched = datums
                .iter()
                .any(|datum| Value::eqv(&key, &datum_to_value(datum)));
            if !matched {
                continue;
            }

            if entries.len() == 1 {
                return Ok(Value::Unspecified);
            }

            if entries.get(1).and_then(|datum| datum.as_symbol()) == Some("=>") {
                if entries.len() != 3 {
                    return Err(SchemeError::syntax(
                        "'case' => clause must contain exactly one recipient expression",
                        None,
                    ));
                }
                let recipient = self.eval_single(entries[2], env.clone(), "'case' recipient")?;
                return self.apply(recipient, env.clone(), vec![key.clone()]);
            }

            return self.eval_begin(&entries[1..], env.clone());
        }

        Ok(Value::Unspecified)
    }

    fn eval_do(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'do' expects bindings, a termination clause, and an optional body",
            ));
        }

        let bindings = parse_do_bindings(items[1])?;
        let termination = items[2]
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("'do' termination clause must be proper", None))?;
        let Some(test_expr) = termination.first() else {
            return Err(SchemeError::syntax(
                "'do' termination clause must contain a test expression",
                None,
            ));
        };
        let result_exprs = &termination[1..];
        let commands = &items[3..];

        let loop_env = Environment::child(env.clone());
        {
            let mut loop_env_mut = loop_env.borrow_mut();
            for (name, init, _) in &bindings {
                let value = self.eval_single(init, env.clone(), "'do' init expression")?;
                loop_env_mut.define(name.clone(), value);
            }
        }

        loop {
            if self
                .eval_single(test_expr, loop_env.clone(), "'do' termination test")?
                .is_truthy()
            {
                if result_exprs.is_empty() {
                    return Ok(Value::Unspecified);
                }
                return self.eval_begin(result_exprs, loop_env.clone());
            }

            if !commands.is_empty() {
                self.eval_begin(commands, loop_env.clone())?;
            }

            let mut updates = Vec::with_capacity(bindings.len());
            for (name, _, step) in &bindings {
                let value = match step {
                    Some(step) => {
                        self.eval_single(step, loop_env.clone(), "'do' step expression")?
                    }
                    None => loop_env.borrow().lookup(name)?,
                };
                updates.push((name.clone(), value));
            }

            for (name, value) in updates {
                loop_env.borrow_mut().set(&name, value)?;
            }
        }
    }

    fn eval_set(&self, items: &[&Datum], env: EnvRef) -> Result<Value, SchemeError> {
        if items.len() != 3 {
            return Err(SchemeError::arity("'set!' expects exactly 2 arguments"));
        }
        let Datum::Symbol(name) = items[1] else {
            return Err(SchemeError::syntax("'set!' target must be a symbol", None));
        };
        let value = self.eval_single(items[2], env.clone(), "'set!' value")?;
        env.borrow_mut().set(name, value)?;
        Ok(Value::Unspecified)
    }

    pub(crate) fn apply(
        &self,
        proc: Value,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<Value, SchemeError> {
        match proc {
            Value::Procedure(proc_ref) => match proc_ref.as_ref() {
                Procedure::Builtin { func, .. } => {
                    self.env_stack.borrow_mut().push(env);
                    let result = func(self, &args);
                    self.env_stack.borrow_mut().pop();
                    result
                }
                Procedure::Native { func, .. } => func(self, env, &args),
                Procedure::Lambda {
                    params,
                    rest,
                    body,
                    env,
                    ..
                } => self.apply_lambda_clause(params, rest, body, env.clone(), args),
                Procedure::CaseLambda { clauses, env, .. } => {
                    let arity = args.len();
                    let clause = clauses.iter().find(|clause| {
                        if let Some(rest) = &clause.rest {
                            let _ = rest;
                            arity >= clause.params.len()
                        } else {
                            arity == clause.params.len()
                        }
                    });

                    let clause = clause.ok_or_else(|| {
                        SchemeError::arity(format!(
                            "case-lambda has no clause matching {} arguments",
                            arity
                        ))
                    })?;
                    self.apply_lambda_clause(
                        &clause.params,
                        &clause.rest,
                        &clause.body,
                        env.clone(),
                        args,
                    )
                }
                Procedure::RecordConstructor { record_type, .. } => {
                    if args.len() != record_type.field_count() {
                        return Err(SchemeError::arity(format!(
                            "record constructor expected {} arguments, got {}",
                            record_type.field_count(),
                            args.len()
                        )));
                    }
                    Ok(Value::record(RecordInstance::new(
                        record_type.clone(),
                        args,
                    )))
                }
                Procedure::RecordPredicate { record_type, .. } => {
                    if args.len() != 1 {
                        return Err(SchemeError::arity(
                            "record predicate expects exactly 1 argument",
                        ));
                    }
                    let is_match = match &args[0] {
                        Value::Record(record) => {
                            let instance_type = record.borrow().record_type();
                            Rc::ptr_eq(&instance_type, record_type)
                        }
                        _ => false,
                    };
                    Ok(Value::Boolean(is_match))
                }
                Procedure::RecordAccessor {
                    record_type,
                    field_index,
                    ..
                } => {
                    if args.len() != 1 {
                        return Err(SchemeError::arity(
                            "record accessor expects exactly 1 argument",
                        ));
                    }
                    let Value::Record(record) = &args[0] else {
                        return Err(SchemeError::type_error(
                            "record accessor expected a record argument",
                        ));
                    };

                    let instance = record.borrow();
                    let instance_type = instance.record_type();
                    if !Rc::ptr_eq(&instance_type, record_type) {
                        return Err(SchemeError::type_error(
                            "record accessor expected a matching record type",
                        ));
                    }

                    instance
                        .field(*field_index)
                        .cloned()
                        .ok_or_else(|| SchemeError::runtime("record accessor index out of range"))
                }
                Procedure::RecordMutator {
                    record_type,
                    field_index,
                    ..
                } => {
                    if args.len() != 2 {
                        return Err(SchemeError::arity(
                            "record mutator expects exactly 2 arguments",
                        ));
                    }
                    let Value::Record(record) = &args[0] else {
                        return Err(SchemeError::type_error(
                            "record mutator expected a record argument",
                        ));
                    };

                    let mut instance = record.borrow_mut();
                    let instance_type = instance.record_type();
                    if !Rc::ptr_eq(&instance_type, record_type) {
                        return Err(SchemeError::type_error(
                            "record mutator expected a matching record type",
                        ));
                    }
                    let Some(true) = record_type.field_mutable(*field_index) else {
                        return Err(SchemeError::runtime(
                            "record field is immutable and cannot be mutated",
                        ));
                    };
                    if !instance.set_field(*field_index, args[1].clone()) {
                        return Err(SchemeError::runtime("record mutator index out of range"));
                    }
                    Ok(Value::Unspecified)
                }
            },
            Value::Parameter(parameter) => match args.as_slice() {
                [] => Ok(parameter.cell().borrow().clone()),
                [value] => {
                    let mut new_value = value.clone();
                    if let Some(converter) = parameter.converter() {
                        new_value = self.apply(converter, self.current_env(), vec![new_value])?;
                    }
                    *parameter.cell().borrow_mut() = new_value;
                    Ok(Value::Unspecified)
                }
                _ => Err(SchemeError::arity(
                    "parameter procedures expect 0 or 1 arguments",
                )),
            },
            Value::Continuation(token) => match args.as_slice() {
                [value] => Err(SchemeError::continuation_jump(token, value.clone())),
                _ => Err(SchemeError::arity(
                    "continuation procedures expect exactly 1 argument",
                )),
            },
            other => Err(SchemeError::type_error(format!(
                "attempted to call a non-procedure: {other}"
            ))),
        }
    }

    fn quasiquote(&self, datum: &Datum, env: EnvRef, depth: usize) -> Result<Value, SchemeError> {
        match datum {
            Datum::Boolean(value) => Ok(Value::Boolean(*value)),
            Datum::Number(value) => Ok(Value::Number(*value)),
            Datum::Character(value) => Ok(Value::Character(*value)),
            Datum::String(value) => Ok(Value::string(value.clone())),
            Datum::Symbol(value) => Ok(Value::symbol(value.clone())),
            Datum::EmptyList => Ok(Value::EmptyList),
            Datum::ByteVector(values) => Ok(Value::bytevector(values.clone())),
            Datum::Vector(values) => self.quasiquote_vector(values, env, depth),
            Datum::Pair(car, cdr) => self.quasiquote_pair(car, cdr, env, depth),
        }
    }

    fn quasiquote_vector(
        &self,
        items: &[Datum],
        env: EnvRef,
        depth: usize,
    ) -> Result<Value, SchemeError> {
        let mut values = Vec::new();
        for item in items {
            if let Some(expr) = extract_abbreviation(item, "unquote-splicing") {
                if depth == 1 {
                    let spliced =
                        self.eval_single(expr, env.clone(), "'unquote-splicing' expression")?;
                    values.extend(expect_quasiquote_splice_list(&spliced)?);
                    continue;
                }
            }
            values.push(self.quasiquote(item, env.clone(), depth)?);
        }
        Ok(Value::vector(values))
    }

    fn quasiquote_pair(
        &self,
        car: &Datum,
        cdr: &Datum,
        env: EnvRef,
        depth: usize,
    ) -> Result<Value, SchemeError> {
        if let Some(expr) = extract_abbreviation_from_pair(car, cdr, "unquote") {
            if depth == 1 {
                return self.eval_single(expr, env, "'unquote' expression");
            }
            return Ok(Value::list(vec![
                Value::symbol("unquote"),
                self.quasiquote(expr, env, depth - 1)?,
            ]));
        }

        if let Some(expr) = extract_abbreviation_from_pair(car, cdr, "quasiquote") {
            return Ok(Value::list(vec![
                Value::symbol("quasiquote"),
                self.quasiquote(expr, env, depth + 1)?,
            ]));
        }

        if let Some(expr) = extract_abbreviation_from_pair(car, cdr, "unquote-splicing") {
            if depth == 1 {
                return Err(SchemeError::syntax(
                    "'unquote-splicing' is only valid inside a list or vector template",
                    None,
                ));
            }
            return Ok(Value::list(vec![
                Value::symbol("unquote-splicing"),
                self.quasiquote(expr, env, depth - 1)?,
            ]));
        }

        if let Some(items) = collect_pair_as_list(car, cdr) {
            let mut values = Vec::new();
            for item in items {
                if let Some(expr) = extract_abbreviation(item, "unquote-splicing") {
                    if depth == 1 {
                        let spliced =
                            self.eval_single(expr, env.clone(), "'unquote-splicing' expression")?;
                        values.extend(expect_quasiquote_splice_list(&spliced)?);
                        continue;
                    }
                }
                values.push(self.quasiquote(item, env.clone(), depth)?);
            }
            return Ok(Value::list(values));
        }

        if extract_abbreviation(car, "unquote-splicing").is_some() && depth == 1 {
            return Err(SchemeError::syntax(
                "'unquote-splicing' is not valid in dotted pair car position",
                None,
            ));
        }

        Ok(Value::pair(
            self.quasiquote(car, env.clone(), depth)?,
            self.quasiquote(cdr, env, depth)?,
        ))
    }

    fn apply_lambda_clause(
        &self,
        params: &[String],
        rest: &Option<String>,
        body: &[Datum],
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<Value, SchemeError> {
        if rest.is_none() && params.len() != args.len() {
            return Err(SchemeError::arity(format!(
                "lambda expected {} arguments, got {}",
                params.len(),
                args.len()
            )));
        }
        if rest.is_some() && args.len() < params.len() {
            return Err(SchemeError::arity(format!(
                "lambda expected at least {} arguments, got {}",
                params.len(),
                args.len()
            )));
        }

        let call_env = Environment::child(env);
        {
            let mut call_env_mut = call_env.borrow_mut();
            for (name, value) in params.iter().zip(args.iter().cloned()) {
                call_env_mut.define(name.clone(), value);
            }
            if let Some(rest_name) = rest {
                let rest_values = args[params.len()..].to_vec();
                call_env_mut.define(rest_name.clone(), Value::list(rest_values));
            }
        }

        let body_refs = body.iter().collect::<Vec<_>>();
        self.eval_body(&body_refs, call_env)
    }
}

fn datum_to_value(datum: &Datum) -> Value {
    Value::from_datum(datum)
}

fn is_definition_form(datum: &Datum) -> bool {
    datum
        .collect_proper_list()
        .and_then(|items| items.first().and_then(|item| item.as_symbol()))
        == Some("define")
}

fn extract_define_name(datum: &Datum) -> Result<String, SchemeError> {
    let items = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("'define' form must be proper", None))?;
    if items.len() < 3 {
        return Err(SchemeError::arity(
            "'define' expects a name and at least one body expression",
        ));
    }

    match items[1] {
        Datum::Symbol(name) => Ok(name.clone()),
        Datum::Pair(_, _) => {
            let signature = items[1].collect_proper_list().ok_or_else(|| {
                SchemeError::syntax("function signature must be a proper list", None)
            })?;
            let Some(Datum::Symbol(name)) = signature.first() else {
                return Err(SchemeError::syntax("function name must be a symbol", None));
            };
            Ok(name.clone())
        }
        _ => Err(SchemeError::syntax(
            "define expects a symbol or function signature",
            None,
        )),
    }
}

fn extract_parameters(items: &[&Datum]) -> Result<Vec<String>, SchemeError> {
    let (params, rest) = extract_formals_from_items(items)?;
    if rest.is_some() {
        return Err(SchemeError::syntax(
            "fixed parameter list expected only symbols",
            None,
        ));
    }
    Ok(params)
}

fn extract_formals(datum: &Datum) -> Result<(Vec<String>, Option<String>), SchemeError> {
    match datum {
        Datum::Symbol(name) => Ok((Vec::new(), Some(name.clone()))),
        Datum::EmptyList => Ok((Vec::new(), None)),
        Datum::Pair(_, _) => {
            let mut params = Vec::new();
            let mut current = datum;
            loop {
                match current {
                    Datum::Pair(car, cdr) => {
                        let Datum::Symbol(name) = car.as_ref() else {
                            return Err(SchemeError::syntax(
                                "parameter list must contain only symbols",
                                None,
                            ));
                        };
                        params.push(name.clone());
                        current = cdr.as_ref();
                    }
                    Datum::EmptyList => return Ok((params, None)),
                    Datum::Symbol(name) => return Ok((params, Some(name.clone()))),
                    _ => {
                        return Err(SchemeError::syntax(
                            "lambda parameter list must be proper or dotted",
                            None,
                        ))
                    }
                }
            }
        }
        _ => Err(SchemeError::syntax(
            "lambda parameter list must be a symbol or list",
            None,
        )),
    }
}

fn extract_formals_from_items(
    items: &[&Datum],
) -> Result<(Vec<String>, Option<String>), SchemeError> {
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
    Ok((params, None))
}

fn extract_abbreviation<'a>(datum: &'a Datum, symbol: &str) -> Option<&'a Datum> {
    let items = datum.collect_proper_list()?;
    extract_abbreviation_from_items(&items, symbol)
}

fn extract_abbreviation_from_pair<'a>(
    car: &'a Datum,
    cdr: &'a Datum,
    symbol: &str,
) -> Option<&'a Datum> {
    let items = collect_pair_as_list(car, cdr)?;
    extract_abbreviation_from_items(&items, symbol)
}

fn extract_abbreviation_from_items<'a>(items: &[&'a Datum], symbol: &str) -> Option<&'a Datum> {
    if items.len() == 2 && items.first().and_then(|item| item.as_symbol()) == Some(symbol) {
        Some(items[1])
    } else {
        None
    }
}

fn expect_quasiquote_splice_list(value: &Value) -> Result<Vec<Value>, SchemeError> {
    value
        .to_proper_list_vec()
        .ok_or_else(|| SchemeError::type_error("'unquote-splicing' expected a proper list result"))
}

fn collect_pair_as_list<'a>(car: &'a Datum, cdr: &'a Datum) -> Option<Vec<&'a Datum>> {
    let mut items = vec![car];
    let mut current = cdr;

    loop {
        match current {
            Datum::EmptyList => return Some(items),
            Datum::Pair(next_car, next_cdr) => {
                items.push(next_car.as_ref());
                current = next_cdr.as_ref();
            }
            _ => return None,
        }
    }
}

#[derive(Clone, Debug)]
struct RecordFieldDecl {
    field_name: String,
    accessor: String,
    mutator: Option<String>,
}

fn parse_record_constructor_spec(datum: &Datum) -> Result<(String, Vec<String>), SchemeError> {
    let items = datum.collect_proper_list().ok_or_else(|| {
        SchemeError::syntax(
            "'define-record-type' constructor spec must be a proper list",
            None,
        )
    })?;

    let Some(Datum::Symbol(constructor_name)) = items.first() else {
        return Err(SchemeError::syntax(
            "'define-record-type' constructor name must be a symbol",
            None,
        ));
    };

    let mut fields = Vec::new();
    for field in &items[1..] {
        let Datum::Symbol(field_name) = field else {
            return Err(SchemeError::syntax(
                "'define-record-type' constructor fields must be symbols",
                None,
            ));
        };
        fields.push(field_name.clone());
    }

    Ok((constructor_name.clone(), fields))
}

fn parse_record_field_specs(items: &[&Datum]) -> Result<Vec<RecordFieldDecl>, SchemeError> {
    let mut fields = Vec::new();
    let mut field_names = HashSet::new();
    let mut proc_names = HashSet::new();

    for item in items {
        let parts = item.collect_proper_list().ok_or_else(|| {
            SchemeError::syntax(
                "'define-record-type' field spec must be a proper list",
                None,
            )
        })?;

        if parts.len() != 2 && parts.len() != 3 {
            return Err(SchemeError::syntax(
                "'define-record-type' field spec must be '(field accessor)' or '(field accessor mutator)'",
                None,
            ));
        }

        let Datum::Symbol(field_name) = parts[0] else {
            return Err(SchemeError::syntax(
                "'define-record-type' field name must be a symbol",
                None,
            ));
        };
        let Datum::Symbol(accessor) = parts[1] else {
            return Err(SchemeError::syntax(
                "'define-record-type' accessor name must be a symbol",
                None,
            ));
        };

        let mutator = if let Some(part) = parts.get(2) {
            let Datum::Symbol(mutator) = part else {
                return Err(SchemeError::syntax(
                    "'define-record-type' mutator name must be a symbol",
                    None,
                ));
            };
            Some(mutator.clone())
        } else {
            None
        };

        if !field_names.insert(field_name.clone()) {
            return Err(SchemeError::syntax(
                "duplicate field name in 'define-record-type'",
                None,
            ));
        }
        if !proc_names.insert(accessor.clone()) {
            return Err(SchemeError::syntax(
                "duplicate accessor/mutator name in 'define-record-type'",
                None,
            ));
        }
        if let Some(mutator) = &mutator {
            if !proc_names.insert(mutator.clone()) {
                return Err(SchemeError::syntax(
                    "duplicate accessor/mutator name in 'define-record-type'",
                    None,
                ));
            }
        }

        fields.push(RecordFieldDecl {
            field_name: field_name.clone(),
            accessor: accessor.clone(),
            mutator,
        });
    }

    Ok(fields)
}

fn parse_bindings(datum: &Datum) -> Result<Vec<(String, &Datum)>, SchemeError> {
    let binding_datums = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("binding list must be proper", None))?;
    let mut bindings = Vec::new();

    for binding in binding_datums {
        let parts = binding
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("binding must be a proper list", None))?;

        if parts.len() != 2 {
            return Err(SchemeError::syntax(
                "each binding must contain exactly a name and an init expression",
                None,
            ));
        }

        let Datum::Symbol(name) = parts[0] else {
            return Err(SchemeError::syntax("binding name must be a symbol", None));
        };

        bindings.push((name.clone(), parts[1]));
    }

    Ok(bindings)
}

fn parse_do_bindings(datum: &Datum) -> Result<Vec<(String, &Datum, Option<&Datum>)>, SchemeError> {
    let binding_datums = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("'do' binding list must be proper", None))?;
    let mut bindings = Vec::new();

    for binding in binding_datums {
        let parts = binding
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("'do' binding must be a proper list", None))?;

        if parts.len() < 2 || parts.len() > 3 {
            return Err(SchemeError::syntax(
                "each 'do' binding must contain a name, init expression, and optional step",
                None,
            ));
        }

        let Datum::Symbol(name) = parts[0] else {
            return Err(SchemeError::syntax(
                "'do' binding name must be a symbol",
                None,
            ));
        };

        bindings.push((name.clone(), parts[1], parts.get(2).copied()));
    }

    Ok(bindings)
}

fn parse_library_name(datum: &Datum) -> Result<String, SchemeError> {
    let items = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("library name must be a proper list", None))?;
    if items.is_empty() {
        return Err(SchemeError::syntax("library name cannot be empty", None));
    }

    let mut parts = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Datum::Symbol(name) => parts.push(name.clone()),
            Datum::Number(number) => parts.push(number.to_string()),
            _ => {
                return Err(SchemeError::syntax(
                    "library name parts must be symbols or numbers",
                    None,
                ));
            }
        }
    }
    Ok(parts.join(" "))
}

fn parse_export_specs(specs: &[&Datum]) -> Result<Vec<(String, String)>, SchemeError> {
    let mut exports = Vec::new();

    for spec in specs {
        match spec {
            Datum::Symbol(name) => exports.push((name.clone(), name.clone())),
            Datum::Pair(_, _) => {
                let parts = spec.collect_proper_list().ok_or_else(|| {
                    SchemeError::syntax("library export spec must be proper", None)
                })?;
                if parts.len() != 3 || parts[0].as_symbol() != Some("rename") {
                    return Err(SchemeError::syntax(
                        "library export rename spec must be '(rename internal external)'",
                        None,
                    ));
                }
                let Datum::Symbol(internal) = parts[1] else {
                    return Err(SchemeError::syntax(
                        "library export rename source must be a symbol",
                        None,
                    ));
                };
                let Datum::Symbol(external) = parts[2] else {
                    return Err(SchemeError::syntax(
                        "library export rename target must be a symbol",
                        None,
                    ));
                };
                exports.push((internal.clone(), external.clone()));
            }
            _ => {
                return Err(SchemeError::syntax(
                    "library export spec must be a symbol or rename form",
                    None,
                ));
            }
        }
    }

    Ok(exports)
}

fn collect_library_exports(
    env: &EnvRef,
    exports: &[(String, String)],
) -> Result<HashMap<String, Value>, SchemeError> {
    let mut bindings = HashMap::new();
    for (internal, external) in exports {
        let value = env.borrow().lookup(internal)?;
        bindings.insert(external.clone(), value);
    }
    Ok(bindings)
}

fn resolve_import_set(datum: &Datum, env: EnvRef) -> Result<HashMap<String, Value>, SchemeError> {
    let items = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("import set must be a proper list", None))?;
    let Some(keyword) = items.first().and_then(|item| item.as_symbol()) else {
        return Err(SchemeError::syntax("import set cannot be empty", None));
    };

    match keyword {
        "only" => {
            if items.len() < 3 {
                return Err(SchemeError::arity(
                    "'only' import set expects a source and at least one identifier",
                ));
            }
            let base = resolve_import_set(items[1], env)?;
            let mut bindings = HashMap::new();
            for item in &items[2..] {
                let Datum::Symbol(name) = item else {
                    return Err(SchemeError::syntax(
                        "'only' import identifiers must be symbols",
                        None,
                    ));
                };
                let value = base.get(name).ok_or_else(|| {
                    SchemeError::name(format!("import set does not contain identifier: {name}"))
                })?;
                bindings.insert(name.clone(), value.clone());
            }
            Ok(bindings)
        }
        "except" => {
            if items.len() < 2 {
                return Err(SchemeError::arity(
                    "'except' import set expects a source import set",
                ));
            }
            let mut base = resolve_import_set(items[1], env)?;
            for item in &items[2..] {
                let Datum::Symbol(name) = item else {
                    return Err(SchemeError::syntax(
                        "'except' import identifiers must be symbols",
                        None,
                    ));
                };
                base.remove(name);
            }
            Ok(base)
        }
        "prefix" => {
            if items.len() != 3 {
                return Err(SchemeError::arity(
                    "'prefix' import set expects a source and a prefix identifier",
                ));
            }
            let base = resolve_import_set(items[1], env)?;
            let Datum::Symbol(prefix) = items[2] else {
                return Err(SchemeError::syntax(
                    "'prefix' import prefix must be a symbol",
                    None,
                ));
            };
            Ok(base
                .into_iter()
                .map(|(name, value)| (format!("{prefix}{name}"), value))
                .collect())
        }
        "rename" => {
            if items.len() < 3 {
                return Err(SchemeError::arity(
                    "'rename' import set expects a source and rename specs",
                ));
            }
            let mut base = resolve_import_set(items[1], env)?;
            let mut renamed = HashMap::new();
            for item in &items[2..] {
                let parts = item.collect_proper_list().ok_or_else(|| {
                    SchemeError::syntax("'rename' import spec must be proper", None)
                })?;
                if parts.len() != 2 {
                    return Err(SchemeError::syntax(
                        "'rename' import spec must contain exactly two identifiers",
                        None,
                    ));
                }
                let Datum::Symbol(from) = parts[0] else {
                    return Err(SchemeError::syntax(
                        "'rename' source identifier must be a symbol",
                        None,
                    ));
                };
                let Datum::Symbol(to) = parts[1] else {
                    return Err(SchemeError::syntax(
                        "'rename' target identifier must be a symbol",
                        None,
                    ));
                };
                let value = base.remove(from).ok_or_else(|| {
                    SchemeError::name(format!("import set does not contain identifier: {from}"))
                })?;
                renamed.insert(to.clone(), value);
            }
            base.extend(renamed);
            Ok(base)
        }
        _ => {
            let library_name = parse_library_name(datum)?;
            env.borrow().lookup_library(&library_name).map_or_else(
                || {
                    Err(SchemeError::name(format!(
                        "unknown library: {library_name}"
                    )))
                },
                |library| Ok(library.bindings().clone()),
            )
        }
    }
}

fn parse_syntax_bindings(datum: &Datum) -> Result<Vec<(String, SyntaxRules)>, SchemeError> {
    let binding_datums = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("syntax binding list must be proper", None))?;
    let mut bindings = Vec::new();

    for binding in binding_datums {
        let parts = binding
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("syntax binding must be a proper list", None))?;

        if parts.len() != 2 {
            return Err(SchemeError::syntax(
                "each syntax binding must contain exactly a name and transformer specification",
                None,
            ));
        }

        let Datum::Symbol(name) = parts[0] else {
            return Err(SchemeError::syntax(
                "syntax binding name must be a symbol",
                None,
            ));
        };

        bindings.push((name.clone(), SyntaxRules::compile(parts[1])?));
    }

    Ok(bindings)
}
