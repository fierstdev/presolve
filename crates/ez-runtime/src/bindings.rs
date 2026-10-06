//! Typed host bindings for `EdgeZero` application and capability interfaces.

pub(crate) mod application {
    wasmtime::component::bindgen!({
        path: "../../wit/edgezero-app",
        world: "application",
    });
}

pub(crate) mod kv {
    wasmtime::component::bindgen!({
        path: "../../wit/edgezero-app",
        interfaces: "
            import edgezero:kv/store@0.1.0;
        ",
    });
}

pub(crate) use application::Application;
