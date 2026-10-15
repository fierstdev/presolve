use semver::Version;

use crate::{CapabilityContract, CapabilityContractError};

/// Canonical capability interface identity for leased work queues.
pub const WORK_QUEUE_INTERFACE: &str = "presolve:queue/work-queue";

/// Publishes one opaque message body to the logical queue.
pub const WORK_QUEUE_OPERATION_SEND: &str = "send";

/// Receives one leased delivery from the logical queue when one is available.
pub const WORK_QUEUE_OPERATION_RECEIVE: &str = "receive";

/// Acknowledges successful processing of one received delivery.
pub const WORK_QUEUE_OPERATION_ACK: &str = "ack";

/// Negatively acknowledges one delivery so it becomes eligible for redelivery.
pub const WORK_QUEUE_OPERATION_RETRY: &str = "retry";

/// Queue message bodies are opaque byte sequences.
///
/// Structured payload formats, schemas, and content types remain application
/// concerns unless a future capability extension standardizes them explicitly.
pub const WORK_QUEUE_GUARANTEE_OPAQUE_BYTE_BODY: &str = "opaque-byte-body";

/// Successfully accepted messages are delivered at least once while they remain
/// within the provider's configured retention and availability boundary.
///
/// Duplicate delivery is therefore valid baseline behavior. Applications must
/// not infer exactly-once processing from this guarantee.
pub const WORK_QUEUE_GUARANTEE_AT_LEAST_ONCE_DELIVERY: &str = "at-least-once-delivery";

/// Received messages are represented as finite in-flight deliveries.
///
/// The provider may use receipt handles, lease identifiers, pending-entry
/// ownership, or another transport-specific mechanism. That mechanism is not
/// application identity and is never part of the canonical queue contract.
pub const WORK_QUEUE_GUARANTEE_LEASED_DELIVERY: &str = "leased-delivery";

/// Successful processing requires an explicit acknowledgement operation.
///
/// Merely receiving a message does not settle that delivery.
pub const WORK_QUEUE_GUARANTEE_EXPLICIT_ACKNOWLEDGEMENT: &str = "explicit-acknowledgement";

/// A delivery that is neither acknowledged nor explicitly retried becomes
/// eligible for redelivery after its provider-defined lease/in-flight window.
pub const WORK_QUEUE_GUARANTEE_UNACKNOWLEDGED_REDELIVERY: &str = "unacknowledged-redelivery";

/// Explicit retry preserves the logical message body while making the message
/// eligible for another delivery attempt.
///
/// Providers may assign a different provider-native message or receipt identity
/// while preserving this application-visible payload semantic.
pub const WORK_QUEUE_GUARANTEE_RETRY_PRESERVES_BODY: &str = "retry-preserves-body";

/// The baseline contract adds no FIFO or processing-order guarantee.
///
/// Applications that require ordered receive behavior must request the
/// corresponding optional feature.
pub const WORK_QUEUE_GUARANTEE_PROVIDER_DEFINED_DELIVERY_ORDER: &str =
    "provider-defined-delivery-order";

/// Provider can defer initial delivery of a newly sent message.
pub const WORK_QUEUE_FEATURE_DELAYED_SEND: &str = "delayed-send";

/// Provider can defer a delivery after explicit retry.
pub const WORK_QUEUE_FEATURE_DELAYED_RETRY: &str = "delayed-retry";

/// Received deliveries expose a provider-maintained delivery-attempt count.
pub const WORK_QUEUE_FEATURE_DELIVERY_ATTEMPT_COUNT: &str = "delivery-attempt-count";

/// Environment can route repeatedly failed messages to a dead-letter queue.
pub const WORK_QUEUE_FEATURE_DEAD_LETTER_ROUTING: &str = "dead-letter-routing";

/// Application may request the lease duration used for a receive operation.
pub const WORK_QUEUE_FEATURE_CONFIGURABLE_LEASE_DURATION: &str = "configurable-lease-duration";

/// Provider preserves enqueue order when making new messages available to
/// receive operations for the configured logical queue.
///
/// This feature does not imply exactly-once processing or completion ordering
/// across concurrently executing consumers.
pub const WORK_QUEUE_FEATURE_ORDERED_RECEIVE: &str = "ordered-receive";

/// Returns the canonical leased work-queue capability contract.
///
/// The baseline models pull-style work distribution: publish opaque payloads,
/// receive finite leased deliveries, explicitly acknowledge success, and
/// explicitly request retry. Workload-trigger topology remains a separate
/// deployment concern rather than part of this provider API.
///
/// # Errors
///
/// Returns an error only if Presolve's built-in static contract definition is
/// internally invalid.
pub fn work_queue_contract() -> Result<CapabilityContract, CapabilityContractError> {
    CapabilityContract::new(
        WORK_QUEUE_INTERFACE,
        Version::new(0, 1, 0),
        [
            WORK_QUEUE_OPERATION_SEND,
            WORK_QUEUE_OPERATION_RECEIVE,
            WORK_QUEUE_OPERATION_ACK,
            WORK_QUEUE_OPERATION_RETRY,
        ],
        [
            WORK_QUEUE_GUARANTEE_OPAQUE_BYTE_BODY,
            WORK_QUEUE_GUARANTEE_AT_LEAST_ONCE_DELIVERY,
            WORK_QUEUE_GUARANTEE_LEASED_DELIVERY,
            WORK_QUEUE_GUARANTEE_EXPLICIT_ACKNOWLEDGEMENT,
            WORK_QUEUE_GUARANTEE_UNACKNOWLEDGED_REDELIVERY,
            WORK_QUEUE_GUARANTEE_RETRY_PRESERVES_BODY,
            WORK_QUEUE_GUARANTEE_PROVIDER_DEFINED_DELIVERY_ORDER,
        ],
        [
            WORK_QUEUE_FEATURE_DELAYED_SEND,
            WORK_QUEUE_FEATURE_DELAYED_RETRY,
            WORK_QUEUE_FEATURE_DELIVERY_ATTEMPT_COUNT,
            WORK_QUEUE_FEATURE_DEAD_LETTER_ROUTING,
            WORK_QUEUE_FEATURE_CONFIGURABLE_LEASE_DURATION,
            WORK_QUEUE_FEATURE_ORDERED_RECEIVE,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_work_queue_contract_is_valid_and_canonical() {
        let contract = work_queue_contract().expect("work queue contract should be valid");

        assert_eq!(contract.interface(), WORK_QUEUE_INTERFACE);
        assert_eq!(contract.version(), &Version::new(0, 1, 0));
        assert_eq!(contract.operations(), ["ack", "receive", "retry", "send"]);
        assert_eq!(
            contract.guarantees(),
            [
                "at-least-once-delivery",
                "explicit-acknowledgement",
                "leased-delivery",
                "opaque-byte-body",
                "provider-defined-delivery-order",
                "retry-preserves-body",
                "unacknowledged-redelivery"
            ]
        );
        assert_eq!(
            contract.optional_features(),
            [
                "configurable-lease-duration",
                "dead-letter-routing",
                "delayed-retry",
                "delayed-send",
                "delivery-attempt-count",
                "ordered-receive"
            ]
        );
    }
}
