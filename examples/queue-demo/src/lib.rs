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
        let expected = input.into_bytes();

        if let Err(error) = work_queue::send(
            &expected,
            work_queue::SendOptions { delay_ms: None },
        ) {
            return format!("error:send:{}", error_code(error));
        }

        let first = match work_queue::receive(work_queue::ReceiveOptions { lease_ms: None }) {
            Ok(Some(delivery)) => delivery,
            Ok(None) => return "error:first-receive-empty".to_owned(),
            Err(error) => return format!("error:first-receive:{}", error_code(error)),
        };

        if first.body() != expected {
            return "error:first-body-mismatch".to_owned();
        }

        if let Err(error) = work_queue::retry(
            first,
            work_queue::RetryOptions { delay_ms: None },
        ) {
            return format!("error:retry:{}", error_code(error));
        }

        let second = match work_queue::receive(work_queue::ReceiveOptions { lease_ms: None }) {
            Ok(Some(delivery)) => delivery,
            Ok(None) => return "error:redelivery-empty".to_owned(),
            Err(error) => return format!("error:redelivery:{}", error_code(error)),
        };

        if second.body() != expected {
            return "error:redelivery-body-mismatch".to_owned();
        }

        if let Err(error) = work_queue::ack(second) {
            return format!("error:ack:{}", error_code(error));
        }

        match work_queue::receive(work_queue::ReceiveOptions { lease_ms: None }) {
            Ok(None) => format!("queue:{}", String::from_utf8_lossy(&expected)),
            Ok(Some(delivery)) => {
                let _ = work_queue::ack(delivery);
                "error:post-ack-visible".to_owned()
            }
            Err(error) => format!("error:post-ack-receive:{}", error_code(error)),
        }
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

bindings::export!(
    QueueComponent with_types_in bindings
);
