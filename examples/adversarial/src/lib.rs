mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/edgezero-app",
        world: "application",
    });
}

struct AdversarialComponent;

impl bindings::Guest for AdversarialComponent {
    fn run(input: String) -> String {
        match input.as_str() {
            "spin" => spin_forever(),
            "memory" => consume_memory(),
            _ => format!("unknown adversarial scenario: {input}"),
        }
    }
}

fn spin_forever() -> ! {
    loop {
        std::hint::black_box(());
    }
}

fn consume_memory() -> String {
    const ALLOCATION_BYTES: usize = 128 * 1024 * 1024;

    let mut allocation = vec![0_u8; ALLOCATION_BYTES];

    for index in (0..allocation.len()).step_by(64 * 1024) {
        allocation[index] = 1;
    }

    allocation.len().to_string()
}

bindings::export!(AdversarialComponent with_types_in bindings);
