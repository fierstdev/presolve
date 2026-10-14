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
        match command(&input) {
            Ok(Command::Set { key, value }) => match store::set(key, value) {
                Ok(()) => "stored".to_owned(),
                Err(error) => format!("error:{}", error_text(error)),
            },
            Ok(Command::Get { key }) => match store::get(key) {
                Ok(Some(value)) => format!("value:{value}"),
                Ok(None) => "missing".to_owned(),
                Err(error) => format!("error:{}", error_text(error)),
            },
            Ok(Command::Delete { key }) => match store::delete(key) {
                Ok(true) => "deleted".to_owned(),
                Ok(false) => "missing".to_owned(),
                Err(error) => format!("error:{}", error_text(error)),
            },
            Err(message) => format!("error:{message}"),
        }
    }
}

enum Command<'a> {
    Set { key: &'a str, value: &'a str },
    Get { key: &'a str },
    Delete { key: &'a str },
}

fn command(input: &str) -> Result<Command<'_>, &'static str> {
    if let Some(rest) = input.strip_prefix("set:") {
        let (key, value) = rest.split_once(':').ok_or("invalid-command")?;
        if key.is_empty() {
            return Err("invalid-command");
        }
        return Ok(Command::Set { key, value });
    }

    if let Some(key) = input.strip_prefix("get:") {
        if key.is_empty() {
            return Err("invalid-command");
        }
        return Ok(Command::Get { key });
    }

    if let Some(key) = input.strip_prefix("delete:") {
        if key.is_empty() {
            return Err("invalid-command");
        }
        return Ok(Command::Delete { key });
    }

    Err("invalid-command")
}

fn error_text(error: store::Error) -> String {
    match error {
        store::Error::Unavailable(message) => format!("unavailable:{message}"),
        store::Error::InvalidKey(message) => format!("invalid-key:{message}"),
        store::Error::Internal(message) => format!("internal:{message}"),
    }
}

bindings::export!(
    KeyValueComponent with_types_in bindings
);
