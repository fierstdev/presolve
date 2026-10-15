mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/presolve-app",
        world: "queue-application",
        with: {
            "presolve:queue/work-queue@0.1.0": generate,
        },
    });
}

use bindings::presolve::queue::work_queue;

struct QueueComponent;

impl bindings::Guest for QueueComponent {
    fn run(input: String) -> String {
        let payload = input.as_bytes();

        if let Err(error) = work_queue::send(
            payload,
            work_queue::SendOptions { delay_ms: None },
        ) {
            return format!("error:send:{}", error_code(error));
        }

        let first = match work_queue::receive(work_queue::ReceiveOptions { lease_ms: None }) {
            Ok(Some(delivery)) => delivery,
            Ok(None) => return "error:first-missing".to_owned(),
            Err(error) => return format!("error:first-receive:{}", error_code(error)),
        };

        if first.body().as_slice() != payload {
            return "error:first-body".to_owned();
        }

        if let Err(error) = work_queue::retry(
            first,
            work_queue::RetryOptions { delay_ms: None },
        ) {
            return format!("error:retry:{}", error_code(error));
        }

        let second = match work_queue::receive(work_queue::ReceiveOptions { lease_ms: None }) {
            Ok(Some(delivery)) => delivery,
            Ok(None) => return "error:retry-missing".to_owned(),
            Err(error) => return format!("error:retry-receive:{}", error_code(error)),
        };

        if second.body().as_slice() != payload {
            return "error:retry-body".to_owned();
        }

        if let Err(error) = work_queue::ack(second) {
            return format!("error:ack:{}", error_code(error));
        }

        format!("queue:{input}")
    }
}

fn error_code(error: work_queue::Error) -> &'static str {
    match error {
        work_queue::Error::Unavailable(_) => "unavailable",
        work_queue::Error::InvalidRequest(_) => "invalid-request",
        work_queue::Error::InvalidDelivery(_) => "invalid-delivery",
        work_queue::Error::Unsupported(_) => "unsupported",
        work_queue::Error::Transient(_) => "transient",
        work_queue::Error::Internal(_) => "internal",
    }
}

bindings::export!(QueueComponent with_types_in bindings);
