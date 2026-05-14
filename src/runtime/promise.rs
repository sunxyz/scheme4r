use std::{cell::RefCell, mem, rc::Rc};

use crate::{error::SchemeError, reader::Datum, runtime::environment::EnvRef, runtime::Value};

pub type PromiseRef = Rc<PromiseObject>;

#[derive(Clone, Debug)]
pub struct PromiseObject {
    state: RefCell<PromiseState>,
}

#[derive(Clone, Debug)]
enum PromiseState {
    Pending {
        expr: Datum,
        env: EnvRef,
        force_result: bool,
    },
    Forcing,
    Forced(Value),
}

#[derive(Clone, Debug)]
pub struct PendingPromise {
    expr: Datum,
    env: EnvRef,
    force_result: bool,
}

impl PromiseObject {
    pub fn new(expr: Datum, env: EnvRef, force_result: bool) -> PromiseRef {
        Rc::new(Self {
            state: RefCell::new(PromiseState::Pending {
                expr,
                env,
                force_result,
            }),
        })
    }

    pub fn forced_value(&self) -> Option<Value> {
        match &*self.state.borrow() {
            PromiseState::Forced(value) => Some(value.clone()),
            _ => None,
        }
    }

    pub fn take_pending(&self) -> Result<PendingPromise, SchemeError> {
        let mut state = self.state.borrow_mut();
        match mem::replace(&mut *state, PromiseState::Forcing) {
            PromiseState::Pending {
                expr,
                env,
                force_result,
            } => Ok(PendingPromise {
                expr,
                env,
                force_result,
            }),
            PromiseState::Forced(value) => {
                *state = PromiseState::Forced(value.clone());
                Err(SchemeError::runtime("promise already forced"))
            }
            PromiseState::Forcing => Err(SchemeError::runtime(
                "promise is already being forced",
            )),
        }
    }

    pub fn restore_pending(&self, pending: PendingPromise) {
        *self.state.borrow_mut() = PromiseState::Pending {
            expr: pending.expr,
            env: pending.env,
            force_result: pending.force_result,
        };
    }

    pub fn store_forced(&self, value: Value) {
        *self.state.borrow_mut() = PromiseState::Forced(value);
    }
}

impl PendingPromise {
    pub fn expr(&self) -> &Datum {
        &self.expr
    }

    pub fn env(&self) -> EnvRef {
        self.env.clone()
    }

    pub fn force_result(&self) -> bool {
        self.force_result
    }
}
