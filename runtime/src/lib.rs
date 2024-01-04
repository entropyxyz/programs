}
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
        config.wasm_component_model(true);
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

        bindings
            .call_evaluate(&mut self.store, signature_request)
            .unwrap()
            .map_err(RuntimeError::Runtime)
    }

    /// Compute the `custom-hash` of a `message` from the program.
    pub fn custom_hash(&mut self, program: &[u8], message: &[u8]) -> Result<[u8; 32], RuntimeError> {
        if program.len() == 0 {
            return Err(RuntimeError::EmptyBytecode);
        }

        let component = Component::from_binary(&self.engine, program)
            .map_err(|_| RuntimeError::InvalidBytecode)?;
        let (bindings, _) = Program::instantiate(&mut self.store, &component, &self.linker)
            .map_err(|_| RuntimeError::InvalidBytecode)?;

        let hash_as_vec = bindings
            .call_custom_hash(&mut self.store, message)
            .unwrap().ok_or(RuntimeError::Runtime(ProgramError::InvalidSignatureRequest("`custom-hash` returns `None`. Implement the hash function in your program, or select a predefined `hash` in your signature request.".to_string())))?;
        if hash_as_vec.len() != 32 {
            return Err(RuntimeError::Runtime(ProgramError::InvalidSignatureRequest(format!("`custom-hash` must returns a Vec<u8> of length 32, not {}.", hash_as_vec.len()))));
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hash_as_vec);
        Ok(hash)
    }
