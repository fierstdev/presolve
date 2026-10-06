use wasmtime::component::Component;

/// A WebAssembly Component compiled for execution by an `EdgeZero` runtime.
///
/// Compilation is intentionally separate from instantiation. Compiling a
/// component is comparatively expensive, while instances and stores are
/// intended to be comparatively short-lived.
pub struct CompiledApplication {
    pub(crate) component: Component,
}

impl CompiledApplication {
    /// Returns the underlying compiled component.
    ///
    /// This is crate-private because `EdgeZero` owns the runtime abstraction.
    pub(crate) const fn component(&self) -> &Component {
        &self.component
    }
}
