mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/edgezero-app",
        world: "application",
    });
}

struct HelloComponent;

impl bindings::Guest for HelloComponent {
    fn run(input: String) -> String {
        format!("Hello from EdgeZero, {input}.")
    }
}

bindings::export!(HelloComponent with_types_in bindings);
