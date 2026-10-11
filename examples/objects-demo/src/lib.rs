mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/presolve-app",
        world: "objects-application",
        with: {
            "presolve:objects/store@0.1.0": generate,
        },
    });
}

use bindings::presolve::objects::store;

struct ObjectStorageComponent;

impl bindings::Guest for ObjectStorageComponent {
    fn run(input: String) -> String {
        const KEY: &str = "demo/message.bin";
        const PREFIX: &str = "demo/";

        let mut expected = vec![0_u8, 1, 2, 255];
        expected.extend_from_slice(input.as_bytes());

        let size = match u64::try_from(expected.len()) {
            Ok(size) => size,
            Err(_) => return "error:body-too-large".to_owned(),
        };

        let write_handle = match store::put(KEY, size) {
            Ok(handle) => handle,
            Err(error) => return format!("error:put:{}", error_code(error)),
        };

        let split = expected.len().min(3);

        if let Err(error) = store::write(write_handle, &expected[..split]) {
            let _ = store::abort_put(write_handle);
            return format!("error:write-first:{}", error_code(error));
        }

        if let Err(error) = store::write(write_handle, &expected[split..]) {
            let _ = store::abort_put(write_handle);
            return format!("error:write-second:{}", error_code(error));
        }

        let committed = match store::finish_put(write_handle) {
            Ok(info) => info,
            Err(error) => return format!("error:finish:{}", error_code(error)),
        };

        if committed.key != KEY || committed.size != size {
            return "error:finish-metadata".to_owned();
        }

        match store::head(KEY) {
            Ok(Some(info)) if info.key == KEY && info.size == size => {}
            Ok(Some(_)) => return "error:head-metadata".to_owned(),
            Ok(None) => return "error:head-missing".to_owned(),
            Err(error) => return format!("error:head:{}", error_code(error)),
        }

        let objects = match store::list_objects(PREFIX) {
            Ok(objects) => objects,
            Err(error) => return format!("error:list:{}", error_code(error)),
        };

        if !objects
            .iter()
            .any(|info| info.key == KEY && info.size == size)
        {
            return "error:list-missing".to_owned();
        }

        let session = match store::get(KEY) {
            Ok(Some(session)) => session,
            Ok(None) => return "error:get-missing".to_owned(),
            Err(error) => return format!("error:get:{}", error_code(error)),
        };

        let store::ReadSession {
            handle: read_handle,
            info: read_info,
        } = session;

        if read_info.key != KEY || read_info.size != size {
            let _ = store::close_read(read_handle);
            return "error:get-metadata".to_owned();
        }

        let mut actual = Vec::new();

        loop {
            let chunk = match store::read(read_handle, 3) {
                Ok(chunk) => chunk,
                Err(error) => {
                    let _ = store::close_read(read_handle);
                    return format!("error:read:{}", error_code(error));
                }
            };

            if chunk.is_empty() {
                break;
            }

            actual.extend_from_slice(&chunk);
        }

        if let Err(error) = store::close_read(read_handle) {
            return format!("error:close-read:{}", error_code(error));
        }

        if actual != expected {
            return "error:body-mismatch".to_owned();
        }

        match store::delete(KEY) {
            Ok(true) => {}
            Ok(false) => return "error:delete-missing".to_owned(),
            Err(error) => return format!("error:delete:{}", error_code(error)),
        }

        match store::head(KEY) {
            Ok(None) => {}
            Ok(Some(_)) => return "error:delete-visible".to_owned(),
            Err(error) => return format!("error:post-delete-head:{}", error_code(error)),
        }

        format!("objects:{input}")
    }
}

fn error_code(error: store::Error) -> &'static str {
    match error {
        store::Error::Unavailable(_) => "unavailable",
        store::Error::InvalidKey(_) => "invalid-key",
        store::Error::InvalidBody(_) => "invalid-body",
        store::Error::InvalidHandle(_) => "invalid-handle",
        store::Error::InvalidRequest(_) => "invalid-request",
        store::Error::Internal(_) => "internal",
    }
}

bindings::export!(
    ObjectStorageComponent with_types_in bindings
);
