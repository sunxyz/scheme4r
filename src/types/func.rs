use crate::env::{Env,RefEnv,EnvOps};

use super::{list::List, types::Type, SExpr};

pub struct ApplyArgs {
    expr: SExpr,
    args: Option<List>,
    lazy_args: fn(List, RefEnv) -> List,
    inter: fn(&Type, RefEnv) -> Type,
    env: RefEnv,
}

impl ApplyArgs {
    pub fn new(
        expr: SExpr,
        args: Option<List>,
        lazy_args: fn(List, RefEnv) -> List,
        inter: fn(&Type, RefEnv) -> Type,
        env: RefEnv,
    ) -> Self {
        ApplyArgs {
            expr,
            args,
            lazy_args,
            inter,
            env,
        }
    }

    pub fn clone_of(&mut self, args: Option<List>) -> ApplyArgs {
        ApplyArgs::new(
            if let Some (l) = args.clone() {l} else {List::new()},
            args,
            |_, _| List::new(),
            self.inter,
            self.env.clone(),
        )
    }

    pub fn fork_env(&mut self) -> ApplyArgs {
        let mut child = self.clone();
        child.env = Env::extend(self.env.clone());
        child
    }

    pub fn expr(&self) -> &List {
        &self.expr
    }

    pub fn args(&mut self) -> &List {
        if let None = self.args {
            let lazy_f = self.lazy_args;
            let v = lazy_f(self.expr().clone(), self.env.clone());
            // println!("args is None exp: {} => {}", self.expr(), v);
            self.args = Some(v);
        }
        self.args.as_ref().unwrap()
    }

    pub fn inter(&mut self, exp: &Type) -> Type {
        let e = self.inter;
        e(exp, self.env.clone())
    }

    pub fn inter_4_env(&mut self, exp: &Type, env: RefEnv) -> Type {
        let e = self.inter;
        e(exp, env)
    }

    pub fn env(&mut self) -> RefEnv {
        self.env.clone()
    }
}

impl Clone for ApplyArgs {
    fn clone(&self) -> Self {
        ApplyArgs::new(
            self.expr.clone(),
            self.args.clone(),
            self.lazy_args,
            self.inter,
            self.env.clone(),
        )
    }
}
