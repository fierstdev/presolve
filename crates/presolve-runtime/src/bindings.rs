//! Typed host bindings for `Presolve` application and capability interfaces.

pub(crate) mod application {
    wasmtime::component::bindgen!({
        path: "../../wit/presolve-app",
        world: "application",
    });
}

pub(crate) mod kv {
    wasmtime::component::bindgen!({
        path: "../../wit/presolve-app",
        interfaces: "
            import presolve:kv/store@0.1.0;
        ",
    });
}

pub(crate) use application::Application;
