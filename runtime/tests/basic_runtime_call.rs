// "barebones" constraints as created by `wasm-tools component new ./barebones.wasm -o barebones_component.wasm`
const BAREBONES_COMPONENT_WASM: &[u8] = include_bytes!("./barebones-component.wasm");

use core::result::Result;

use wasmtime::{
    component::{bindgen, Component, Linker},
    Caller, Config, Engine, Func, Instance, Module, Store,
};

bindgen!("constraint");

#[test]
fn test_barebones_component() {
    // Create a new engine supporting the component model
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();

    // let barebones_module = Module::new(&engine, BAREBONES_COMPONENT_WASM).unwrap();
    let component = Component::from_binary(&engine, BAREBONES_COMPONENT_WASM).unwrap();

    let linker = Linker::new(&engine);
    // let instance = linker.instantiate(&mut store, &barebones_module).unwrap();

    let mut store = Store::new(&engine, ());
    // let evaluate = instance.get_typed_func::<(), (Result<(), ec_core::bindgen::Error>)>(&mut store, "evaluate").unwrap();

    let rand_stuff = "asdfasdfasdfasdf".to_string();
    let initial_state = EvaluationState {
        data: rand_stuff.as_bytes(),
    };

    let (bindings, _) = Constraint::instantiate(&mut store, &component, &linker).unwrap();
    // let res = evaluate(&mut store, initial_state).unwrap();
    // evaluate.call(&mut store, input_data);
    let res = bindings.call_evaluate(&mut store, initial_state);
    assert!(res.unwrap().is_ok());
}

#[test]
fn test_barebones_component_failure() {
    // Create a new engine supporting the component model
    let mut config = Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config).unwrap();

    // let barebones_module = Module::new(&engine, BAREBONES_COMPONENT_WASM).unwrap();
    let component = Component::from_binary(&engine, BAREBONES_COMPONENT_WASM).unwrap();

    let linker = Linker::new(&engine);
    // let instance = linker.instantiate(&mut store, &barebones_module).unwrap();

    let mut store = Store::new(&engine, ());
    // let evaluate = instance.get_typed_func::<(), (Result<(), ec_core::bindgen::Error>)>(&mut store, "evaluate").unwrap();

    let rand_stuff = "asdff".to_string();
    let initial_state = EvaluationState {
        data: rand_stuff.as_bytes(),
    };

    let (bindings, _) = Constraint::instantiate(&mut store, &component, &linker).unwrap();
    // let res = evaluate(&mut store, initial_state).unwrap();
    // evaluate.call(&mut store, input_data);
    let res = bindings.call_evaluate(&mut store, initial_state);
    assert!(res.unwrap().is_err());
}

// This is basically the example from wasmtime docs

#[test]
fn test_default_wasmtime_example() {
    // Modules can be compiled through either the text or binary format
    let engine = Engine::default();
    let wat = r#"
        (module
            (import "host" "host_func" (func $host_hello (param i32)))

            (func (export "hello")
                i32.const 3
                call $host_hello)
        )
    "#;
    let module = Module::new(&engine, wat).unwrap();

    // All wasm objects operate within the context of a "store". Each
    // `Store` has a type parameter to store host-specific data, which in
    // this case we're using `4` for.
    let mut store = Store::new(&engine, 4);
    let host_func = Func::wrap(&mut store, |caller: Caller<'_, u32>, param: i32| {
        println!("Got {} from WebAssembly", param);
        println!("my host state is: {}", caller.data());
    });

    // Instantiation of a module requires specifying its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    let instance = Instance::new(&mut store, &module, &[host_func.into()]).unwrap();
    let hello = instance
        .get_typed_func::<(), ()>(&mut store, "hello")
        .unwrap();

    // And finally we can call the wasm!
    hello.call(&mut store, ()).unwrap();
}
