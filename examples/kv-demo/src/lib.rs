mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/presolve-app",
        world: "kv-application",
        with: {
            "presolve:kv/store@0.1.0": generate,
        },
    });
}

use bindings::presolve::kv::store;

struct KeyValueComponent;

impl bindings::Guest for KeyValueComponent {
    fn run(input: String) -> String {
        const KEY: &str = "message";

        if let Err(error) = store::set(KEY, &input) {
            return format!("error:{}", error_code(error));
        }

        let value = match store::get(KEY) {
            Ok(Some(value)) => value,
            Ok(None) => {
                return "error:missing".to_owned();
            }
            Err(error) => {
                return format!("error:{}", error_code(error));
            }
        };

        match store::delete(KEY) {
            Ok(true) => {
                format!("stored:{value}")
            }
            Ok(false) => "error:delete-missing".to_owned(),
            Err(error) => {
                format!("error:{}", error_code(error))
            }
        }
    }
}

fn error_code(error: store::Error) -> &'static str {
    match error {
        store::Error::Unavailable(_) => "unavailable",
        store::Error::InvalidKey(_) => "invalid-key",
        store::Error::Internal(_) => "internal",
    }
}

bindings::export!(
    KeyValueComponent with_types_in bindings
);
