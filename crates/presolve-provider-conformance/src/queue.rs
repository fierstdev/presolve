use std::{
    thread,
    time::{Duration, Instant},
};

use presolve_provider_sdk::{
    WorkQueueDelivery, WorkQueueProvider, WorkQueueReceiveOptions, WorkQueueRetryOptions,
    WorkQueueSendOptions,
};

/// Timing bounds used by the baseline work-queue conformance suite.
///
/// The provider fixture should configure an isolated queue with an in-flight
/// lease/recovery interval comfortably below `redelivery_timeout`. The harness
/// intentionally uses baseline receive options so configurable lease duration
/// is not required for conformance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkQueueConformanceOptions {
    redelivery_timeout: Duration,
    poll_interval: Duration,
}

impl WorkQueueConformanceOptions {
    /// Creates conformance timing bounds.
    #[must_use]
    pub const fn new(redelivery_timeout: Duration, poll_interval: Duration) -> Self {
        Self {
            redelivery_timeout,
            poll_interval,
        }
    }

    /// Returns the maximum time allowed to observe a required redelivery.
    #[must_use]
    pub const fn redelivery_timeout(self) -> Duration {
        self.redelivery_timeout
    }

    /// Returns the delay between empty receive polls.
    #[must_use]
    pub const fn poll_interval(self) -> Duration {
        self.poll_interval
    }
}

/// Exercises mandatory `presolve:queue/work-queue@0.1.0` provider semantics.
///
/// The caller must supply an isolated queue with no unrelated application
/// messages. At-least-once delivery means duplicates are legal, so the harness
/// never asserts that acknowledgement creates a global exactly-once guarantee
/// and never assumes FIFO ordering.
///
/// The harness verifies:
///
/// - opaque binary and empty bodies survive send/receive unchanged;
/// - deliveries can be explicitly acknowledged;
/// - dropping an unacknowledged delivery permits observable redelivery;
/// - explicit retry preserves the logical message body.
///
/// This function panics on semantic violations and is intended for provider
/// test suites/checkpoints.
pub fn assert_work_queue_conformance(
    provider: &dyn WorkQueueProvider,
    options: WorkQueueConformanceOptions,
) {
    assert!(!options.redelivery_timeout().is_zero());
    assert!(!options.poll_interval().is_zero());
    assert!(options.poll_interval() <= options.redelivery_timeout());

    assert_body_round_trip(provider, options, b"presolve-queue-baseline\0\x01\xff");
    assert_body_round_trip(provider, options, b"");
    assert_unacknowledged_redelivery(provider, options);
    assert_explicit_retry(provider, options);
}

fn assert_body_round_trip(
    provider: &dyn WorkQueueProvider,
    options: WorkQueueConformanceOptions,
    body: &[u8],
) {
    provider
        .send(body, WorkQueueSendOptions::default())
        .expect("baseline queue send should succeed");

    let delivery = receive_matching(provider, options, body);
    delivery
        .ack()
        .expect("baseline queue acknowledgement should succeed");
}

fn assert_unacknowledged_redelivery(
    provider: &dyn WorkQueueProvider,
    options: WorkQueueConformanceOptions,
) {
    let body = b"presolve-queue-unacknowledged-redelivery";
    provider
        .send(body, WorkQueueSendOptions::default())
        .expect("redelivery test send should succeed");

    let delivery = receive_matching(provider, options, body);
    drop(delivery);

    let redelivery = receive_matching(provider, options, body);
    redelivery
        .ack()
        .expect("redelivered message acknowledgement should succeed");
}

fn assert_explicit_retry(provider: &dyn WorkQueueProvider, options: WorkQueueConformanceOptions) {
    let body = b"presolve-queue-explicit-retry";
    provider
        .send(body, WorkQueueSendOptions::default())
        .expect("retry test send should succeed");

    let delivery = receive_matching(provider, options, body);
    delivery
        .retry(WorkQueueRetryOptions::default())
        .expect("baseline queue retry should succeed");

    let retried = receive_matching(provider, options, body);
    retried
        .ack()
        .expect("retried message acknowledgement should succeed");
}

fn receive_matching(
    provider: &dyn WorkQueueProvider,
    options: WorkQueueConformanceOptions,
    expected_body: &[u8],
) -> Box<dyn WorkQueueDelivery> {
    let deadline = Instant::now() + options.redelivery_timeout();

    loop {
        let delivery = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("baseline queue receive should succeed");

        if let Some(delivery) = delivery {
            if delivery.body() == expected_body {
                return delivery;
            }

            // At-least-once providers may surface duplicates from earlier
            // harness messages. Settle those known-isolated leftovers rather
            // than accidentally turning duplicate tolerance into FIFO/exactly-
            // once assertions.
            delivery
                .ack()
                .expect("duplicate harness delivery should remain acknowledgeable");
        }

        assert!(
            Instant::now() < deadline,
            "expected queue body was not delivered within the conformance timeout"
        );
        thread::sleep(options.poll_interval());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_options_round_trip() {
        let timeout = Duration::from_secs(2);
        let poll = Duration::from_millis(10);
        let options = WorkQueueConformanceOptions::new(timeout, poll);

        assert_eq!(options.redelivery_timeout(), timeout);
        assert_eq!(options.poll_interval(), poll);
    }
}
