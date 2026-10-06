//! Typed host bindings for the `EdgeZero` application world.

wasmtime::component::bindgen!({
    path: "../../wit/edgezero-app",
    world: "application",
});
