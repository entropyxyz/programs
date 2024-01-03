//! Contains the Wasm runtime and related types for evaluating programs.

use thiserror::Error;
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Result, Store,
};

/// Note, this is wasmtime's bindgen, not wit-bindgen (modules)
mod bindgen {
    use super::bindgen;

    bindgen!({
        world: "program",
        path: "../wit/application.wit"
    });
}
pub use bindgen::{Error as ProgramError, Program, SignatureRequest};

/// Runtime `Error` type
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Program bytecode is of zero length (core-side runtime error; Programs should probably not return this)
    #[error("Bytecode length is zero")]
    EmptyBytecode,
    /// Program bytecode is not a valid WebAssembly component.
    #[error("Invalid bytecode")]
    InvalidBytecode,
    /// Runtime error during execution.
    #[error("Runtime error: {0}")]
    Runtime(ProgramError),
}

/// Runtime allows for the execution of programs. Instantiate with `Runtime::new()`.
pub struct Runtime {
    engine: Engine,
    linker: Linker<()>,
    store: Store<()>,
}

impl Default for Runtime {
    fn default() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true).consume_fuel(true);
        let engine = Engine::new(&config).unwrap();
        let linker = Linker::new(&engine);
        let store = Store::new(&engine, ());
        Self {
            engine,
            linker,
            store,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Runtime {
    /// Evaluate a program with a given initial state.
    pub fn evaluate(
        &mut self,
        program: &[u8],
        signature_request: &SignatureRequest,
    ) -> Result<(), RuntimeError> {
        if program.len() == 0 {
            return Err(RuntimeError::EmptyBytecode);
        }

        let component = Component::from_binary(&self.engine, program)
            .map_err(|_| RuntimeError::InvalidBytecode)?;

        let (bindings, _) = Program::instantiate(&mut self.store, &component, &self.linker)
            .map_err(|_| RuntimeError::InvalidBytecode)?;

        // TODO fix this unwrap
        bindings
            .call_evaluate(&mut self.store, signature_request)
            .unwrap()
            .map_err(RuntimeError::Runtime)
    }
}
