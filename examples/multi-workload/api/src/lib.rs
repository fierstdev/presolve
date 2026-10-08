mod bindings {
    wit_bindgen::generate!({
        path: "../../../wit/presolve-app",
        world: "application",
    });
}

struct ApiComponent;

impl bindings::Guest for ApiComponent {
    fn run(input: String) -> String {
        format!("api:{input}")
    }
}

bindings::export!(ApiComponent with_types_in bindings);
