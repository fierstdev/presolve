use presolve_provider_sdk::{KeyValueError, KeyValueErrorCode};

use crate::{bindings::kv::presolve::kv::store, state::RuntimeState};

impl store::Host for RuntimeState {
    fn get(&mut self, key: String) -> Result<Option<String>, store::Error> {
        self.key_value_provider()
            .get(&key)
            .map_err(|error| map_error(&error))
    }

    fn set(&mut self, key: String, value: String) -> Result<(), store::Error> {
        self.key_value_provider()
            .set(&key, &value)
            .map_err(|error| map_error(&error))
    }

    fn delete(&mut self, key: String) -> Result<bool, store::Error> {
        self.key_value_provider()
            .delete(&key)
            .map_err(|error| map_error(&error))
    }
}

fn map_error(error: &KeyValueError) -> store::Error {
    let message = error.message().to_owned();

    match error.code() {
        KeyValueErrorCode::Unavailable => store::Error::Unavailable(message),
        KeyValueErrorCode::InvalidKey => store::Error::InvalidKey(message),
        KeyValueErrorCode::Internal => store::Error::Internal(message),
    }
}
