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
        procedure::{LambdaClause, Procedure},
        EnvRef, Environment, Library, Port, PortRef, RecordFieldSpec, RecordInstance, RecordType,
        Value,
    },
};

#[derive(Clone)]
struct PortState {
    current_input: PortRef,
    current_output: PortRef,
    current_error: PortRef,
}

enum EvalStep {
    Done(Value),
    Eval(Datum, EnvRef),
    Apply(Value, EnvRef, Vec<Value>),
    CallWithCurrentContinuation(Value, EnvRef),
    DynamicWind {
        before: Value,
        thunk: Value,
        after: Value,
        env: EnvRef,
    },
    WithExceptionHandler {
        handler: Value,
        thunk: Value,
        env: EnvRef,
    },
}

enum DriverAction {
    Step(EvalStep),
    Return(Value),
    Raise(SchemeError),
}

enum ControlFrame {
    CallWithCurrentContinuation {
        token: usize,
    },
    DynamicWind {
        after: Value,
        env: EnvRef,
        stage: DynamicWindStage,
    },
    WithExceptionHandler {
        handler: Value,
        env: EnvRef,
        stage: ExceptionHandlerStage,
    },
}

enum DynamicWindStage {
    Before { thunk: Value },
    Thunk,
    AfterValue(Value),
    AfterError(SchemeError),
}

enum ExceptionHandlerStage {
    Thunk,
    HandlerResult { continuable: bool },
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

    pub(crate) fn build_environment_from_import_sets(
        &self,
        args: &[Value],
    ) -> Result<EnvRef, SchemeError> {
        if args.is_empty() {
            return Err(SchemeError::arity(
                "'environment' expects at least 1 import set",
            ));
        }

        let env = Environment::isolated_with_registry(self.root_env.clone());
        let mut imported = HashMap::new();
        for arg in args {
            let datum = arg.to_datum()?;
            imported.extend(resolve_import_set(&datum, env.clone())?);
        }
        env.borrow_mut().import_bindings(&imported);
        Ok(env)
    }

    pub(crate) fn expect_environment_specifier(
        &self,
        value: &Value,
        name: &str,
    ) -> Result<EnvRef, SchemeError> {
        match value {
            Value::Environment(env) => Ok(env.clone()),
            other => Err(SchemeError::type_error(format!(
                "'{name}' expected an environment specifier, got {other}"
            ))),
        }
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
        self.drive_step(EvalStep::Eval(expr.clone(), env))
    }

    fn drive_step(&self, initial_step: EvalStep) -> Result<Value, SchemeError> {
        let mut action = DriverAction::Step(initial_step);
        let mut frames = Vec::new();

        loop {
            action = match action {
                DriverAction::Step(step) => match step {
                    EvalStep::Done(value) => DriverAction::Return(value),
                    EvalStep::Eval(expr, env) => match self.eval_step(&expr, env) {
                        Ok(step) => DriverAction::Step(step),
                        Err(err) => DriverAction::Raise(err),
                    },
                    EvalStep::Apply(proc, env, args) => match self.apply_step(proc, env, args) {
                        Ok(step) => DriverAction::Step(step),
                        Err(err) => DriverAction::Raise(err),
                    },
                    EvalStep::CallWithCurrentContinuation(receiver, env) => {
                        let token = next_driver_continuation_token();
                        frames.push(ControlFrame::CallWithCurrentContinuation { token });
                        DriverAction::Step(EvalStep::Apply(
                            receiver,
                            env,
                            vec![Value::Continuation(token)],
                        ))
                    }
                    EvalStep::DynamicWind {
                        before,
                        thunk,
                        after,
                        env,
                    } => {
                        frames.push(ControlFrame::DynamicWind {
                            after,
                            env: env.clone(),
                            stage: DynamicWindStage::Before { thunk },
                        });
                        DriverAction::Step(EvalStep::Apply(before, env, Vec::new()))
                    }
                    EvalStep::WithExceptionHandler { handler, thunk, env } => {
                        frames.push(ControlFrame::WithExceptionHandler {
                            handler,
                            env: env.clone(),
                            stage: ExceptionHandlerStage::Thunk,
                        });
                        DriverAction::Step(EvalStep::Apply(thunk, env, Vec::new()))
                    }
                },
                DriverAction::Return(value) => match self.handle_driver_value(value, &mut frames) {
                    DriverAction::Return(value) => return Ok(value),
                    next => next,
                },
                DriverAction::Raise(err) => match self.handle_driver_error(err, &mut frames) {
                    DriverAction::Raise(err) => return Err(err),
                    next => next,
                },
            };
        }
    }

    fn handle_driver_value(&self, mut value: Value, frames: &mut Vec<ControlFrame>) -> DriverAction {
        loop {
            let Some(frame) = frames.pop() else {
                return DriverAction::Return(value);
            };

            match frame {
                ControlFrame::CallWithCurrentContinuation { .. } => {}
                ControlFrame::DynamicWind { after, env, stage } => match stage {
                    DynamicWindStage::Before { thunk } => {
                        frames.push(ControlFrame::DynamicWind {
                            after,
                            env: env.clone(),
                            stage: DynamicWindStage::Thunk,
                        });
                        return DriverAction::Step(EvalStep::Apply(thunk, env, Vec::new()));
                    }
                    DynamicWindStage::Thunk => {
                        frames.push(ControlFrame::DynamicWind {
                            after: after.clone(),
                            env: env.clone(),
                            stage: DynamicWindStage::AfterValue(value),
                        });
                        return DriverAction::Step(EvalStep::Apply(after, env, Vec::new()));
                    }
                    DynamicWindStage::AfterValue(stored) => {
                        value = stored;
                    }
                    DynamicWindStage::AfterError(err) => {
                        return DriverAction::Raise(err);
                    }
                },
                ControlFrame::WithExceptionHandler { stage, .. } => match stage {
                    ExceptionHandlerStage::Thunk => {}
                    ExceptionHandlerStage::HandlerResult { continuable } => {
                        if continuable {
                            return DriverAction::Return(value);
                        }
                        return DriverAction::Raise(SchemeError::raised(value, false));
                    }
                },
            }
        }
    }

    fn handle_driver_error(&self, err: SchemeError, frames: &mut Vec<ControlFrame>) -> DriverAction {
        loop {
            let Some(frame) = frames.pop() else {
                return DriverAction::Raise(err);
            };

            match frame {
                ControlFrame::CallWithCurrentContinuation { token } => {
                    if let Some((jump_token, value)) = err.as_continuation_jump() {
                        if jump_token == token {
                            return self.handle_driver_value(value.clone(), frames);
                        }
                    }
                }
                ControlFrame::DynamicWind { after, env, stage } => match stage {
                    DynamicWindStage::Before { .. } => {}
                    DynamicWindStage::Thunk => {
                        frames.push(ControlFrame::DynamicWind {
                            after: after.clone(),
                            env: env.clone(),
                            stage: DynamicWindStage::AfterError(err),
                        });
                        return DriverAction::Step(EvalStep::Apply(after, env, Vec::new()));
                    }
                    DynamicWindStage::AfterValue(_) | DynamicWindStage::AfterError(_) => {}
                },
                ControlFrame::WithExceptionHandler {
                    handler,
                    env,
                    stage,
                } => match stage {
                    ExceptionHandlerStage::Thunk => {
                        if let Some((object, continuable)) = err.as_raised() {
                            frames.push(ControlFrame::WithExceptionHandler {
                                handler: handler.clone(),
                                env: env.clone(),
                                stage: ExceptionHandlerStage::HandlerResult { continuable },
                            });
                            return DriverAction::Step(EvalStep::Apply(
                                handler,
                                env,
                                vec![object.clone()],
                            ));
                        }
                    }
                    ExceptionHandlerStage::HandlerResult { .. } => {}
                },
            }
        }
    }

    fn eval_step(&self, expr: &Datum, env: EnvRef) -> Result<EvalStep, SchemeError> {
        match expr {
            Datum::Boolean(value) => Ok(EvalStep::Done(Value::Boolean(*value))),
            Datum::Number(value) => Ok(EvalStep::Done(Value::Number(*value))),
            Datum::Character(value) => Ok(EvalStep::Done(Value::Character(*value))),
            Datum::String(value) => Ok(EvalStep::Done(Value::string(value.clone()))),
            Datum::Vector(values) => Ok(EvalStep::Done(Value::vector(
                values.iter().map(datum_to_value).collect(),
            ))),
            Datum::ByteVector(values) => Ok(EvalStep::Done(Value::bytevector(values.clone()))),
            Datum::Symbol(name) => Ok(EvalStep::Done(env.borrow().lookup(name)?)),
            Datum::EmptyList => Ok(EvalStep::Done(Value::EmptyList)),
            Datum::Pair(_, _) => self.eval_list_form(expr, env),
        }
    }

    fn eval_list_form(&self, expr: &Datum, env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                "define-values" => return self.eval_define_values(&items, env),
                "define-record-type" => return self.eval_define_record_type(&items, env),
                "define-syntax" => return self.eval_define_syntax(&items, env),
                "import" => return self.eval_import(&items[1..], env),
                "define-library" => return self.eval_define_library(&items, env),
                "lambda" => return self.eval_lambda(&items, env, None),
                "case-lambda" => return self.eval_case_lambda(&items, env, None),
                "begin" => return self.eval_begin(&items[1..], env),
                "set!" => return self.eval_set(&items, env),
                "and" => return self.eval_and(&items[1..], env),
                "or" => return self.eval_or(&items[1..], env),
                "let" => return self.eval_let(&items, env),
                "let*" => return self.eval_let_star(&items, env),
                "letrec" => return self.eval_letrec(&items, env),
                "let-values" => return self.eval_let_values(&items, env),
                "let*-values" => return self.eval_let_star_values(&items, env),
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
                return Ok(EvalStep::Eval(expanded, env));
            }
        }

        let operator = self.eval_single(first, env.clone(), "procedure position")?;
        let mut args = Vec::new();
        for expr in &items[1..] {
            args.push(self.eval_single(expr, env.clone(), "argument position")?);
        }
        Ok(EvalStep::Apply(operator, env, args))
    }

    fn eval_quote(&self, items: &[&Datum]) -> Result<EvalStep, SchemeError> {
        if items.len() != 2 {
            return Err(SchemeError::arity("'quote' expects exactly 1 argument"));
        }
        Ok(EvalStep::Done(datum_to_value(items[1])))
    }

    fn eval_quasiquote(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        if items.len() != 2 {
            return Err(SchemeError::arity(
                "'quasiquote' expects exactly 1 argument",
            ));
        }
        Ok(EvalStep::Done(self.quasiquote(items[1], env, 1)?))
    }

    fn eval_if(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        if items.len() < 3 || items.len() > 4 {
            return Err(SchemeError::arity(
                "'if' expects 2 or 3 arguments after the keyword",
            ));
        }

        let predicate = self.eval_single(items[1], env.clone(), "'if' predicate")?;
        if predicate.is_truthy() {
            Ok(EvalStep::Eval(items[2].clone(), env))
        } else if items.len() == 4 {
            Ok(EvalStep::Eval(items[3].clone(), env))
        } else {
            Ok(EvalStep::Done(Value::Unspecified))
        }
    }

    fn eval_define(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                Ok(EvalStep::Done(Value::Unspecified))
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
                Ok(EvalStep::Done(Value::Unspecified))
            }
            _ => Err(SchemeError::syntax(
                "define expects a symbol or function signature",
                None,
            )),
        }
    }

    fn eval_define_values(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        if items.len() != 3 {
            return Err(SchemeError::arity(
                "'define-values' expects a formals list and exactly 1 value expression",
            ));
        }

        let (params, rest) = extract_formals(items[1])?;
        let values = unpack_values(self.eval(items[2], env.clone())?);
        {
            let mut env_mut = env.borrow_mut();
            bind_formals_values(
                &mut env_mut,
                &params,
                &rest,
                values,
                "define-values",
                BindingMode::Define,
            )?;
        }
        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_define_record_type(
        &self,
        items: &[&Datum],
        env: EnvRef,
    ) -> Result<EvalStep, SchemeError> {
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

        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_internal_define(&self, datum: &Datum, env: EnvRef) -> Result<(), SchemeError> {
        let items = datum
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("'define' form must be proper", None))?;

        match items.first().and_then(|item| item.as_symbol()) {
            Some("define") => match items[1] {
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
            },
            Some("define-values") => {
                if items.len() != 3 {
                    return Err(SchemeError::arity(
                        "'define-values' expects a formals list and exactly 1 value expression",
                    ));
                }

                let (params, rest) = extract_formals(items[1])?;
                let values = unpack_values(self.eval(items[2], env.clone())?);
                let mut env_mut = env.borrow_mut();
                bind_formals_values(
                    &mut env_mut,
                    &params,
                    &rest,
                    values,
                    "define-values",
                    BindingMode::Set,
                )
            }
            _ => Err(SchemeError::syntax("unsupported definition form", None)),
        }
    }

    fn eval_lambda(
        &self,
        items: &[&Datum],
        env: EnvRef,
        name: Option<String>,
    ) -> Result<EvalStep, SchemeError> {
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
        Ok(EvalStep::Done(Value::lambda(name, params, rest, body, env)))
    }

    fn eval_case_lambda(
        &self,
        items: &[&Datum],
        env: EnvRef,
        name: Option<String>,
    ) -> Result<EvalStep, SchemeError> {
        if items.len() < 2 {
            return Err(SchemeError::arity(
                "'case-lambda' expects at least 1 clause",
            ));
        }

        let mut clauses = Vec::new();
        for clause in &items[1..] {
            let parts = clause.collect_proper_list().ok_or_else(|| {
                SchemeError::syntax("'case-lambda' clauses must be proper lists", None)
            })?;
            if parts.len() < 2 {
                return Err(SchemeError::syntax(
                    "'case-lambda' clauses must contain formals and at least 1 body expression",
                    None,
                ));
            }
            let (params, rest) = extract_formals(parts[0])?;
            let body = parts[1..]
                .iter()
                .map(|datum| (*datum).clone())
                .collect::<Vec<_>>();
            clauses.push(LambdaClause { params, rest, body });
        }

        Ok(EvalStep::Done(Value::case_lambda(name, clauses, env)))
    }

    fn eval_define_syntax(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_import(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_define_library(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
            let step = self.eval_begin(&begin_forms, library_env.clone())?;
            self.drive_step(step)?;
        }

        let exported_bindings = collect_library_exports(&library_env, &exports)?;
        env.borrow_mut()
            .define_library(library_name, Library::new(exported_bindings));
        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_let_syntax(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        self.eval_local_syntax(items, env, false)
    }

    fn eval_letrec_syntax(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        self.eval_local_syntax(items, env, true)
    }

    fn eval_local_syntax(
        &self,
        items: &[&Datum],
        env: EnvRef,
        _recursive: bool,
    ) -> Result<EvalStep, SchemeError> {
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

    fn eval_begin(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        let Some((last, rest)) = items.split_last() else {
            return Ok(EvalStep::Done(Value::Unspecified));
        };

        for expr in rest {
            self.eval(expr, env.clone())?;
        }
        Ok(EvalStep::Eval((*last).clone(), env))
    }

    fn eval_body(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                for name in extract_definition_names(define)? {
                    body_env_mut.define(name, Value::Unspecified);
                }
            }
        }

        for define in &items[..define_count] {
            self.eval_internal_define(define, body_env.clone())?;
        }

        self.eval_begin(&items[define_count..], body_env)
    }

    fn eval_and(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        let mut result = Value::Boolean(true);
        for expr in items {
            result = self.eval_single(expr, env.clone(), "'and' expression")?;
            if !result.is_truthy() {
                return Ok(EvalStep::Done(result));
            }
        }
        Ok(EvalStep::Done(result))
    }

    fn eval_or(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        for expr in items {
            let value = self.eval_single(expr, env.clone(), "'or' expression")?;
            if value.is_truthy() {
                return Ok(EvalStep::Done(value));
            }
        }
        Ok(EvalStep::Done(Value::Boolean(false)))
    }

    fn eval_let(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                Ok(EvalStep::Apply(proc, let_env, args))
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

    fn eval_let_star(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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

    fn eval_letrec(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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

    fn eval_let_values(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'let-values' expects bindings and at least 1 body expression",
            ));
        }

        let bindings = parse_value_bindings(items[1])?;
        let let_env = Environment::child(env.clone());
        let mut resolved = Vec::with_capacity(bindings.len());
        for (formals, init) in bindings {
            let (params, rest) = extract_formals(formals)?;
            let values = unpack_values(self.eval(init, env.clone())?);
            resolved.push((params, rest, values));
        }

        {
            let mut let_env_mut = let_env.borrow_mut();
            for (params, rest, values) in resolved {
                bind_formals_values(
                    &mut let_env_mut,
                    &params,
                    &rest,
                    values,
                    "let-values",
                    BindingMode::Define,
                )?;
            }
        }

        self.eval_body(&items[2..], let_env)
    }

    fn eval_let_star_values(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        if items.len() < 3 {
            return Err(SchemeError::arity(
                "'let*-values' expects bindings and at least 1 body expression",
            ));
        }

        let bindings = parse_value_bindings(items[1])?;
        let let_env = Environment::child(env);
        for (formals, init) in bindings {
            let (params, rest) = extract_formals(formals)?;
            let values = unpack_values(self.eval(init, let_env.clone())?);
            let mut let_env_mut = let_env.borrow_mut();
            bind_formals_values(
                &mut let_env_mut,
                &params,
                &rest,
                values,
                "let*-values",
                BindingMode::Define,
            )?;
        }

        self.eval_body(&items[2..], let_env)
    }

    fn eval_cond(&self, clauses: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                return Ok(EvalStep::Done(test_value));
            }

            if entries.get(1).and_then(|datum| datum.as_symbol()) == Some("=>") {
                if entries.len() != 3 {
                    return Err(SchemeError::syntax(
                        "'cond' => clause must contain exactly one recipient expression",
                        None,
                    ));
                }
                let recipient = self.eval_single(entries[2], env.clone(), "'cond' recipient")?;
                return Ok(EvalStep::Apply(recipient, env.clone(), vec![test_value]));
            }

            return self.eval_begin(&entries[1..], env.clone());
        }

        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_case(&self, clauses: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                return Ok(EvalStep::Done(Value::Unspecified));
            }

            if entries.get(1).and_then(|datum| datum.as_symbol()) == Some("=>") {
                if entries.len() != 3 {
                    return Err(SchemeError::syntax(
                        "'case' => clause must contain exactly one recipient expression",
                        None,
                    ));
                }
                let recipient = self.eval_single(entries[2], env.clone(), "'case' recipient")?;
                return Ok(EvalStep::Apply(recipient, env.clone(), vec![key.clone()]));
            }

            return self.eval_begin(&entries[1..], env.clone());
        }

        Ok(EvalStep::Done(Value::Unspecified))
    }

    fn eval_do(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
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
                    return Ok(EvalStep::Done(Value::Unspecified));
                }
                return self.eval_begin(result_exprs, loop_env.clone());
            }

            if !commands.is_empty() {
                let step = self.eval_begin(commands, loop_env.clone())?;
                self.drive_step(step)?;
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

    fn eval_set(&self, items: &[&Datum], env: EnvRef) -> Result<EvalStep, SchemeError> {
        if items.len() != 3 {
            return Err(SchemeError::arity("'set!' expects exactly 2 arguments"));
        }
        let Datum::Symbol(name) = items[1] else {
            return Err(SchemeError::syntax("'set!' target must be a symbol", None));
        };
        let value = self.eval_single(items[2], env.clone(), "'set!' value")?;
        env.borrow_mut().set(name, value)?;
        Ok(EvalStep::Done(Value::Unspecified))
    }

    pub(crate) fn apply(
        &self,
        proc: Value,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<Value, SchemeError> {
        self.drive_step(EvalStep::Apply(proc, env, args))
    }

    fn apply_step(
        &self,
        proc: Value,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<EvalStep, SchemeError> {
        match proc {
            Value::Procedure(proc_ref) => match proc_ref.as_ref() {
                Procedure::Builtin { name, func } => match name.as_str() {
                    "apply" => self.apply_apply_builtin(env, args),
                    "call-with-values" => self.apply_call_with_values_builtin(env, args),
                    "call-with-current-continuation" | "call/cc" => {
                        self.apply_call_with_current_continuation_builtin(env, args)
                    }
                    "dynamic-wind" => self.apply_dynamic_wind_builtin(env, args),
                    "with-exception-handler" => {
                        self.apply_with_exception_handler_builtin(env, args)
                    }
                    "eval" => self.apply_eval_builtin(env, args),
                    _ => {
                        self.env_stack.borrow_mut().push(env);
                        let result = func(self, &args);
                        self.env_stack.borrow_mut().pop();
                        Ok(EvalStep::Done(result?))
                    }
                },
                Procedure::Native { func, .. } => Ok(EvalStep::Done(func(self, env, &args)?)),
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
                    Ok(EvalStep::Done(Value::record(RecordInstance::new(
                        record_type.clone(),
                        args,
                    ))))
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
                    Ok(EvalStep::Done(Value::Boolean(is_match)))
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

                    Ok(EvalStep::Done(
                        instance
                        .field(*field_index)
                        .cloned()
                        .ok_or_else(|| SchemeError::runtime("record accessor index out of range"))?,
                    ))
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
                    Ok(EvalStep::Done(Value::Unspecified))
                }
            },
            Value::Parameter(parameter) => match args.as_slice() {
                [] => Ok(EvalStep::Done(parameter.cell().borrow().clone())),
                [value] => {
                    let mut new_value = value.clone();
                    if let Some(converter) = parameter.converter() {
                        new_value = self.apply(converter, self.current_env(), vec![new_value])?;
                    }
                    *parameter.cell().borrow_mut() = new_value;
                    Ok(EvalStep::Done(Value::Unspecified))
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

    fn apply_apply_builtin(&self, env: EnvRef, args: Vec<Value>) -> Result<EvalStep, SchemeError> {
        if args.len() < 2 {
            return Err(SchemeError::arity(
                "'apply' expects a procedure, optional arguments, and a final list",
            ));
        }

        let procedure = args[0].clone();
        let (last, leading) = args[1..].split_last().ok_or_else(|| {
            SchemeError::arity("'apply' expects a procedure and a final argument list")
        })?;

        let mut applied_args = leading.to_vec();
        applied_args.extend(last.to_proper_list_vec().ok_or_else(|| {
            SchemeError::type_error(format!("'apply' expected a proper list, got {last}"))
        })?);
        Ok(EvalStep::Apply(procedure, env, applied_args))
    }

    fn apply_call_with_values_builtin(
        &self,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<EvalStep, SchemeError> {
        if args.len() != 2 {
            return Err(SchemeError::arity(format!(
                "'call-with-values' expects {} arguments, got {}",
                2,
                args.len()
            )));
        }

        let producer = args[0].clone();
        let consumer = args[1].clone();
        let produced = self.apply(producer, env.clone(), Vec::new())?;
        let consumer_args = match produced {
            Value::Multiple(values) => values,
            value => vec![value],
        };
        Ok(EvalStep::Apply(consumer, env, consumer_args))
    }

    fn apply_call_with_current_continuation_builtin(
        &self,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<EvalStep, SchemeError> {
        if args.len() != 1 {
            return Err(SchemeError::arity(format!(
                "'call-with-current-continuation' expects {} arguments, got {}",
                1,
                args.len()
            )));
        }

        Ok(EvalStep::CallWithCurrentContinuation(args[0].clone(), env))
    }

    fn apply_dynamic_wind_builtin(
        &self,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<EvalStep, SchemeError> {
        if args.len() != 3 {
            return Err(SchemeError::arity(format!(
                "'dynamic-wind' expects {} arguments, got {}",
                3,
                args.len()
            )));
        }

        Ok(EvalStep::DynamicWind {
            before: args[0].clone(),
            thunk: args[1].clone(),
            after: args[2].clone(),
            env,
        })
    }

    fn apply_with_exception_handler_builtin(
        &self,
        env: EnvRef,
        args: Vec<Value>,
    ) -> Result<EvalStep, SchemeError> {
        if args.len() != 2 {
            return Err(SchemeError::arity(format!(
                "'with-exception-handler' expects {} arguments, got {}",
                2,
                args.len()
            )));
        }

        Ok(EvalStep::WithExceptionHandler {
            handler: args[0].clone(),
            thunk: args[1].clone(),
            env,
        })
    }

    fn apply_eval_builtin(&self, env: EnvRef, args: Vec<Value>) -> Result<EvalStep, SchemeError> {
        if args.is_empty() || args.len() > 2 {
            return Err(SchemeError::arity(format!(
                "'eval' expects 1 or 2 arguments, got {}",
                args.len()
            )));
        }

        let target_env = match args.get(1) {
            Some(value) => self.expect_environment_specifier(value, "eval")?,
            None => env,
        };

        match &args[0] {
            Value::String(source) => {
                self.eval_forms_as_step(
                    Reader::new(&source.to_plain_string()).read_all()?,
                    target_env,
                )
            }
            value => Ok(EvalStep::Eval(value.to_datum()?, target_env)),
        }
    }

    fn eval_forms_as_step(&self, forms: Vec<Datum>, env: EnvRef) -> Result<EvalStep, SchemeError> {
        let Some((last, rest)) = forms.split_last() else {
            return Ok(EvalStep::Done(Value::Unspecified));
        };

        for form in rest {
            self.eval(form, env.clone())?;
        }
        Ok(EvalStep::Eval(last.clone(), env))
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
    ) -> Result<EvalStep, SchemeError> {
        let call_env = Environment::child(env);
        {
            let mut call_env_mut = call_env.borrow_mut();
            bind_formals_values(
                &mut call_env_mut,
                params,
                rest,
                args,
                "lambda",
                BindingMode::Define,
            )?;
        }

        let body_refs = body.iter().collect::<Vec<_>>();
        self.eval_body(&body_refs, call_env)
    }
}

fn datum_to_value(datum: &Datum) -> Value {
    Value::from_datum(datum)
}

fn is_definition_form(datum: &Datum) -> bool {
    matches!(
        datum.collect_proper_list()
            .and_then(|items| items.first().and_then(|item| item.as_symbol())),
        Some("define" | "define-values")
    )
}

fn extract_definition_names(datum: &Datum) -> Result<Vec<String>, SchemeError> {
    let items = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("'define' form must be proper", None))?;
    if items.len() < 3 {
        return Err(SchemeError::arity(
            "'define' expects a name and at least one body expression",
        ));
    }

    match items.first().and_then(|item| item.as_symbol()) {
        Some("define") => match items[1] {
            Datum::Symbol(name) => Ok(vec![name.clone()]),
            Datum::Pair(_, _) => {
                let signature = items[1].collect_proper_list().ok_or_else(|| {
                    SchemeError::syntax("function signature must be a proper list", None)
                })?;
                let Some(Datum::Symbol(name)) = signature.first() else {
                    return Err(SchemeError::syntax("function name must be a symbol", None));
                };
                Ok(vec![name.clone()])
            }
            _ => Err(SchemeError::syntax(
                "define expects a symbol or function signature",
                None,
            )),
        },
        Some("define-values") => {
            if items.len() != 3 {
                return Err(SchemeError::arity(
                    "'define-values' expects a formals list and exactly 1 value expression",
                ));
            }
            let (params, rest) = extract_formals(items[1])?;
            let mut names = params;
            if let Some(rest_name) = rest {
                names.push(rest_name);
            }
            Ok(names)
        }
        _ => Err(SchemeError::syntax(
            "unsupported definition form",
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

fn parse_value_bindings(datum: &Datum) -> Result<Vec<(&Datum, &Datum)>, SchemeError> {
    let binding_datums = datum
        .collect_proper_list()
        .ok_or_else(|| SchemeError::syntax("value binding list must be proper", None))?;
    let mut bindings = Vec::new();

    for binding in binding_datums {
        let parts = binding
            .collect_proper_list()
            .ok_or_else(|| SchemeError::syntax("value binding must be a proper list", None))?;

        if parts.len() != 2 {
            return Err(SchemeError::syntax(
                "each value binding must contain exactly formals and an init expression",
                None,
            ));
        }

        bindings.push((parts[0], parts[1]));
    }

    Ok(bindings)
}

#[derive(Clone, Copy)]
enum BindingMode {
    Define,
    Set,
}

fn unpack_values(value: Value) -> Vec<Value> {
    match value {
        Value::Multiple(values) => values,
        value => vec![value],
    }
}

fn bind_formals_values(
    env: &mut Environment,
    params: &[String],
    rest: &Option<String>,
    values: Vec<Value>,
    context: &str,
    mode: BindingMode,
) -> Result<(), SchemeError> {
    if rest.is_none() && params.len() != values.len() {
        return Err(SchemeError::arity(format!(
            "'{context}' expected {} values, got {}",
            params.len(),
            values.len()
        )));
    }
    if rest.is_some() && values.len() < params.len() {
        return Err(SchemeError::arity(format!(
            "'{context}' expected at least {} values, got {}",
            params.len(),
            values.len()
        )));
    }

    for (name, value) in params.iter().zip(values.iter().cloned()) {
        match mode {
            BindingMode::Define => env.define(name.clone(), value),
            BindingMode::Set => env.set(name, value)?,
        }
    }

    if let Some(rest_name) = rest {
        let rest_value = Value::list(values[params.len()..].to_vec());
        match mode {
            BindingMode::Define => env.define(rest_name.clone(), rest_value),
            BindingMode::Set => env.set(rest_name, rest_value)?,
        }
    }

    Ok(())
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

fn next_driver_continuation_token() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_TOKEN: AtomicUsize = AtomicUsize::new(1);
    NEXT_TOKEN.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_x(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
        if !args.is_empty() {
            return Err(SchemeError::arity("'current-x' expects exactly 0 arguments"));
        }
        engine.current_env().borrow().lookup("x")
    }

    #[test]
    fn apply_builtin_preserves_current_env_for_nested_builtin_calls() {
        let env = Environment::standard();
        env.borrow_mut()
            .define("current-x", Value::builtin("current-x", current_x));
        let engine = Engine::new(env);

        let value = engine.run("(let ((x 42)) (apply current-x '()))").unwrap();
        assert!(matches!(value, Value::Number(42)));
    }

    #[test]
    fn call_with_values_builtin_preserves_current_env_for_consumer_calls() {
        let env = Environment::standard();
        env.borrow_mut()
            .define("current-x", Value::builtin("current-x", current_x));
        let engine = Engine::new(env);

        let value = engine
            .run(
                "\
                (let ((x 42))
                  (call-with-values
                    (lambda () (values))
                    (lambda () (current-x))))
                ",
            )
            .unwrap();
        assert!(matches!(value, Value::Number(42)));
    }

    #[test]
    fn eval_builtin_preserves_current_env_for_nested_builtin_calls() {
        let env = Environment::standard();
        env.borrow_mut()
            .define("current-x", Value::builtin("current-x", current_x));
        let engine = Engine::new(env);

        let value = engine.run("(let ((x 42)) (eval '(current-x)))").unwrap();
        assert!(matches!(value, Value::Number(42)));
    }

    #[test]
    fn call_cc_builtin_preserves_current_env_for_receiver_calls() {
        let env = Environment::standard();
        env.borrow_mut()
            .define("current-x", Value::builtin("current-x", current_x));
        let engine = Engine::new(env);

        let value = engine
            .run("(let ((x 42)) (call/cc (lambda (k) (current-x))))")
            .unwrap();
        assert!(matches!(value, Value::Number(42)));
    }

    #[test]
    fn dynamic_wind_builtin_preserves_current_env_for_thunk_calls() {
        let env = Environment::standard();
        env.borrow_mut()
            .define("current-x", Value::builtin("current-x", current_x));
        let engine = Engine::new(env);

        let value = engine
            .run(
                "\
                (let ((x 42))
                  (dynamic-wind
                    (lambda () 'before)
                    (lambda () (current-x))
                    (lambda () 'after)))
                ",
            )
            .unwrap();
        assert!(matches!(value, Value::Number(42)));
    }

    #[test]
    fn with_exception_handler_builtin_preserves_current_env_for_handler_calls() {
        let env = Environment::standard();
        env.borrow_mut()
            .define("current-x", Value::builtin("current-x", current_x));
        let engine = Engine::new(env);

        let value = engine
            .run(
                "\
                (let ((x 42))
                  (with-exception-handler
                    (lambda (obj) (current-x))
                    (lambda () (raise 'boom))))
                ",
            )
            .unwrap_err();

        let (object, continuable) = value.as_raised().unwrap();
        assert!(!continuable);
        assert!(matches!(object, Value::Number(42)));
    }
}
