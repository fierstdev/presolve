mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/presolve-app",
        world: "application",
    });
}

struct HelloComponent;

impl bindings::Guest for HelloComponent {
    fn run(input: String) -> String {
        format!("Hello from Presolve, {input}.")
    }
}

bindings::export!(HelloComponent with_types_in bindings);
