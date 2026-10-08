mod bindings {
    wit_bindgen::generate!({
        path: "../../../wit/presolve-app",
        world: "application",
    });
}

struct WorkerComponent;

impl bindings::Guest for WorkerComponent {
    fn run(input: String) -> String {
        format!("worker:{input}")
    }
}

bindings::export!(WorkerComponent with_types_in bindings);
