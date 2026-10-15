use std::time::Duration;

pub use presolve_capability::WORK_QUEUE_INTERFACE;
use thiserror::Error;

/// Stable categories for work-queue provider failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkQueueErrorCode {
    /// The provider cannot currently service requests.
    Unavailable,

    /// The message body or requested operation options are invalid.
    InvalidRequest,

    /// The delivery is no longer valid for settlement.
    InvalidDelivery,

    /// The selected provider does not implement a requested optional semantic.
    Unsupported,

    /// The operation failed for a condition that callers may reasonably retry.
    Transient,

    /// The provider encountered an internal failure.
    Internal,
}

/// Failure returned by a work-queue provider.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct WorkQueueError {
    code: WorkQueueErrorCode,
    message: String,
}

impl WorkQueueError {
    /// Creates an unavailable-provider error.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(WorkQueueErrorCode::Unavailable, message)
    }

    /// Creates an invalid-request error.
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(WorkQueueErrorCode::InvalidRequest, message)
    }

    /// Creates an invalid-delivery error.
    #[must_use]
    pub fn invalid_delivery(message: impl Into<String>) -> Self {
        Self::new(WorkQueueErrorCode::InvalidDelivery, message)
    }

    /// Creates an unsupported-semantic error.
    #[must_use]
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(WorkQueueErrorCode::Unsupported, message)
    }

    /// Creates a transient provider error.
    #[must_use]
    pub fn transient(message: impl Into<String>) -> Self {
        Self::new(WorkQueueErrorCode::Transient, message)
    }

    /// Creates an internal-provider error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(WorkQueueErrorCode::Internal, message)
    }

    fn new(code: WorkQueueErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Returns the stable provider error category.
    #[must_use]
    pub const fn code(&self) -> WorkQueueErrorCode {
        self.code
    }

    /// Returns the human-readable provider error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Options for publishing one queue message.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkQueueSendOptions {
    delay: Option<Duration>,
}

impl WorkQueueSendOptions {
    /// Creates baseline send options with no requested delay.
    #[must_use]
    pub const fn new() -> Self {
        Self { delay: None }
    }

    /// Requests delayed initial delivery.
    ///
    /// The selected provider must advertise `delayed-send` when this is set.
    #[must_use]
    pub const fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Returns the requested initial-delivery delay.
    #[must_use]
    pub const fn delay(self) -> Option<Duration> {
        self.delay
    }
}

/// Options for receiving one queue delivery.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkQueueReceiveOptions {
    lease_duration: Option<Duration>,
}

impl WorkQueueReceiveOptions {
    /// Creates baseline receive options using provider/environment lease policy.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lease_duration: None,
        }
    }

    /// Requests a delivery lease duration.
    ///
    /// The selected provider must advertise `configurable-lease-duration` when
    /// this is set.
    #[must_use]
    pub const fn with_lease_duration(mut self, lease_duration: Duration) -> Self {
        self.lease_duration = Some(lease_duration);
        self
    }

    /// Returns the requested delivery lease duration.
    #[must_use]
    pub const fn lease_duration(self) -> Option<Duration> {
        self.lease_duration
    }
}

/// Options for explicitly retrying one delivery.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WorkQueueRetryOptions {
    delay: Option<Duration>,
}

impl WorkQueueRetryOptions {
    /// Creates baseline retry options with no requested delay.
    #[must_use]
    pub const fn new() -> Self {
        Self { delay: None }
    }

    /// Requests delayed retry.
    ///
    /// The selected provider must advertise `delayed-retry` when this is set.
    #[must_use]
    pub const fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    /// Returns the requested retry delay.
    #[must_use]
    pub const fn delay(self) -> Option<Duration> {
        self.delay
    }
}

/// One finite in-flight work-queue delivery.
///
/// The object owns the provider-specific receipt/lease state without exposing
/// that state to application code. Successful settlement consumes the object,
/// structurally preventing double acknowledgement or retry.
///
/// Dropping a delivery without calling [`Self::ack`] or [`Self::retry`] must not
/// acknowledge it. The provider remains responsible for making that logical
/// message eligible for redelivery according to its lease/recovery semantics.
pub trait WorkQueueDelivery: Send {
    /// Returns the opaque application message body.
    #[must_use]
    fn body(&self) -> &[u8];

    /// Returns a provider-maintained delivery-attempt count when available.
    ///
    /// `None` is valid baseline behavior. Providers should return `Some` only
    /// when the negotiated `delivery-attempt-count` feature applies.
    #[must_use]
    fn delivery_attempt(&self) -> Option<u32> {
        None
    }

    /// Acknowledges successful processing and consumes this delivery.
    ///
    /// # Errors
    ///
    /// Returns [`WorkQueueError`] when the provider cannot settle the delivery.
    fn ack(self: Box<Self>) -> Result<(), WorkQueueError>;

    /// Requests redelivery of the logical message and consumes this delivery.
    ///
    /// # Errors
    ///
    /// Returns [`WorkQueueError`] when retry cannot be requested or when the
    /// supplied options require unsupported semantics.
    fn retry(self: Box<Self>, options: WorkQueueRetryOptions) -> Result<(), WorkQueueError>;
}

/// Host implementation of `presolve:queue/work-queue@0.1.0`.
pub trait WorkQueueProvider: Send + Sync {
    /// Publishes one opaque message body.
    ///
    /// # Errors
    ///
    /// Returns [`WorkQueueError`] when the message cannot be accepted or when
    /// the supplied options require unsupported semantics.
    fn send(&self, body: &[u8], options: WorkQueueSendOptions) -> Result<(), WorkQueueError>;

    /// Receives at most one finite leased delivery.
    ///
    /// `Ok(None)` means no message is currently available. It is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`WorkQueueError`] when receive fails or when the supplied
    /// options require unsupported semantics.
    fn receive(
        &self,
        options: WorkQueueReceiveOptions,
    ) -> Result<Option<Box<dyn WorkQueueDelivery>>, WorkQueueError>;
}

/// Provider used when no work-queue capability implementation is configured.
#[derive(Debug, Default)]
pub struct UnavailableWorkQueueProvider;

impl WorkQueueProvider for UnavailableWorkQueueProvider {
    fn send(&self, _body: &[u8], _options: WorkQueueSendOptions) -> Result<(), WorkQueueError> {
        Err(WorkQueueError::unavailable(
            "work queue capability is not configured",
        ))
    }

    fn receive(
        &self,
        _options: WorkQueueReceiveOptions,
    ) -> Result<Option<Box<dyn WorkQueueDelivery>>, WorkQueueError> {
        Err(WorkQueueError::unavailable(
            "work queue capability is not configured",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_builders_preserve_requested_values() {
        let delay = Duration::from_secs(7);
        let lease = Duration::from_secs(12);

        assert_eq!(
            WorkQueueSendOptions::new().with_delay(delay).delay(),
            Some(delay)
        );
        assert_eq!(
            WorkQueueReceiveOptions::new()
                .with_lease_duration(lease)
                .lease_duration(),
            Some(lease)
        );
        assert_eq!(
            WorkQueueRetryOptions::new().with_delay(delay).delay(),
            Some(delay)
        );
    }

    #[test]
    fn unavailable_provider_reports_stable_category() {
        let error = UnavailableWorkQueueProvider
            .send(b"message", WorkQueueSendOptions::default())
            .expect_err("unavailable queue should reject send");

        assert_eq!(error.code(), WorkQueueErrorCode::Unavailable);
    }
}
