use std::fmt;

use crate::{
    runtime::{FuncAddr, ModuleAddr},
    Value,
};

#[derive(Clone, PartialEq)]
pub struct StackTrace(Vec<StackFrame>);

impl StackTrace {
    pub fn new(frames: Vec<StackFrame>) -> StackTrace {
        StackTrace(frames)
    }

    pub fn frames(&self) -> &[StackFrame] {
        &self.0
    }
}

#[derive(Clone, PartialEq)]
pub struct StackFrame {
    module: ModuleAddr,
    func: Option<FuncAddr>,
}

impl StackFrame {
    pub fn new(module: ModuleAddr, func: Option<FuncAddr>) -> StackFrame {
        StackFrame { module, func }
    }

    pub fn module(&self) -> ModuleAddr {
        self.module
    }

    pub fn func(&self) -> Option<FuncAddr> {
        self.func
    }
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(func) = self.func {
            write!(f, "0x{:08X}", func)
        } else {
            write!(f, "<module: 0x{:08X}>", self.module)
        }
    }
}

impl fmt::Debug for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Represents the context under which a function executes.
///
/// The execution context contains the following items:
/// * The operand stack for the invocation.
/// * The values of the locals currently in scope.
/// * A [`StackFrame`] representing the current location in the program.
pub struct ExecutionContext {
    values: Vec<Value>,
    locals: Vec<Value>,
    frame: StackFrame,
}

impl ExecutionContext {
    /// Creates a new execution context with the specified [`ExecutionContext`] and a list of local values.
    pub fn new(frame: StackFrame, locals: Vec<Value>) -> ExecutionContext {
        ExecutionContext {
            values: Vec::new(),
            frame,
            locals,
        }
    }

    /// Gets the [`StackFrame`] associated with this execution context.
    pub fn frame(&self) -> &StackFrame {
        &self.frame
    }

    /// Pushes a new value on to the operand stack for this execution context.
    pub fn push(&mut self, value: Value) {
        // Don't push nils, just drop them.
        if value != Value::Nil {
            self.values.push(value)
        }
    }

    /// Pops a new value off the operand stack for this execution context.
    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    /// Gets a boolean indicating if the operand stack for this execution context is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Gets the value of the local with the specified index.
    pub fn local(&self, idx: usize) -> Option<Value> {
        if idx < self.locals.len() {
            Some(self.locals[idx])
        } else {
            None
        }
    }
}

pub struct ExecutionStack(Vec<ExecutionContext>);

impl ExecutionStack {
    pub fn new() -> ExecutionStack {
        ExecutionStack(Vec::new())
    }

    /// Gets a reference to the active [`ExecutionContext`]
    ///
    /// # Panics
    /// Panics if there is no current [`ExecutionContext`] on the stack
    pub fn current(&self) -> &ExecutionContext {
        self.0.last().unwrap()
    }

    /// Gets a mutable reference to the active [`ExecutionContext`].
    ///
    /// # Panics
    /// Panics if there is no current [`ExecutionContext`] on the stack
    pub fn current_mut(&mut self) -> &mut ExecutionContext {
        self.0.last_mut().unwrap()
    }

    /// Pushes a new [`ExecutionContext`] on to the stack
    pub fn enter(&mut self, module: ModuleAddr, func: Option<FuncAddr>, locals: Vec<Value>) {
        self.0
            .push(ExecutionContext::new(StackFrame::new(module, func), locals))
    }

    /// Pops the current [`ExecutionContext`] (and all values associated with it) off the stack
    ///
    /// # Panics
    /// Panics if there is no current [`ExecutionContext`] on the stack
    pub fn exit(&mut self) {
        if self.0.len() == 1 {
            panic!("There is no current frame to exit!");
        } else {
            self.0.pop();
        }
    }

    /// Creates a [`StackTrace`] representing the current position in the stack.
    pub fn trace(&self) -> StackTrace {
        // Iterate up the stack from bottom to top, cloning the stack frames
        let frames = self.0.iter().rev().map(|c| c.frame().clone()).collect();
        StackTrace(frames)
    }
}
