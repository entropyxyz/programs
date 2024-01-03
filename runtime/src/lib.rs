//! Contains the Wasm runtime and related types for evaluating programs.

use thiserror::Error;
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config as WasmtimeConfig, Engine, Result, Store,
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

/// Config is for runtime parameters (eg instructions per program, additional runtime interfaces, etc).
pub struct Config {
    /// Max number of instructions that will execute before the runtime returns an error.
    pub max_instructions_per_program: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_instructions_per_program: 10_000,
        }
    }
}

/// Runtime allows for the execution of programs. Instantiate with `Runtime::new()`.
pub struct Runtime {
    engine: Engine,
    linker: Linker<()>,
    store: Store<()>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Runtime {
    pub fn new(config: Config) -> Self {
        let mut wasmtime_config = WasmtimeConfig::new();
        wasmtime_config.wasm_component_model(true).consume_fuel(true);

        let engine = Engine::new(&wasmtime_config).unwrap();
        let linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());

        store.add_fuel(config.max_instructions_per_program).unwrap();
        Self {
            engine,
            linker,
            store,
        }
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
