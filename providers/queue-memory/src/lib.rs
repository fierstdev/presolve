#![forbid(unsafe_code)]

//! In-memory implementation of `presolve:queue/work-queue@0.1.0`.
//!
//! This provider exists for local development, deterministic semantic tests,
//! and as a reference implementation of Presolve's leased-delivery ownership
//! model. It is not intended to provide durable process-external storage.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use presolve_provider_sdk::{
    WorkQueueDelivery, WorkQueueError, WorkQueueProvider, WorkQueueReceiveOptions,
    WorkQueueRetryOptions, WorkQueueSendOptions,
};

/// Node-side adapter identity used for the in-memory work queue.
pub const MEMORY_QUEUE_ADAPTER_ID: &str = "memory:queue";

const DEFAULT_LEASE_DURATION: Duration = Duration::from_secs(30);

#[derive(Debug)]
struct Message {
    id: u64,
    body: Vec<u8>,
    lease_until: Option<Instant>,
    generation: u64,
}

impl Message {
    fn is_available(&self, now: Instant) -> bool {
        self.lease_until.is_none_or(|deadline| deadline <= now)
    }
}

#[derive(Debug, Default)]
struct QueueState {
    next_id: u64,
    messages: VecDeque<Message>,
}

/// Process-local leased work queue implementing only the mandatory 0.1
/// semantics.
///
/// Optional operation parameters are rejected unless a future realization
/// explicitly negotiates and configures those features. This keeps runtime
/// behavior aligned with the resolver-selected capability contract.
#[derive(Debug, Clone)]
pub struct InMemoryWorkQueueProvider {
    state: Arc<Mutex<QueueState>>,
    default_lease_duration: Duration,
}

impl Default for InMemoryWorkQueueProvider {
    fn default() -> Self {
        Self::with_default_lease(DEFAULT_LEASE_DURATION)
    }
}

impl InMemoryWorkQueueProvider {
    /// Creates an empty queue with the default 30-second delivery lease.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty queue with an environment-selected baseline lease.
    ///
    /// A zero-duration lease is useful for deterministic semantic tests because
    /// an abandoned delivery becomes eligible on the next receive without a
    /// wall-clock sleep. This is provider/environment configuration, not the
    /// application-facing `configurable-lease-duration` feature.
    #[must_use]
    pub fn with_default_lease(default_lease_duration: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(QueueState::default())),
            default_lease_duration,
        }
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, QueueState>, WorkQueueError> {
        self.state
            .lock()
            .map_err(|_| WorkQueueError::internal("in-memory queue lock was poisoned"))
    }
}

impl WorkQueueProvider for InMemoryWorkQueueProvider {
    fn send(&self, body: &[u8], options: WorkQueueSendOptions) -> Result<(), WorkQueueError> {
        if options.delay().is_some() {
            return Err(WorkQueueError::unsupported(
                "memory queue realization did not negotiate `delayed-send`",
            ));
        }

        let mut state = self.lock_state()?;
        let id = state.next_id;
        state.next_id = state
            .next_id
            .checked_add(1)
            .ok_or_else(|| WorkQueueError::internal("in-memory queue message id overflow"))?;
        state.messages.push_back(Message {
            id,
            body: body.to_vec(),
            lease_until: None,
            generation: 0,
        });
        Ok(())
    }

    fn receive(
        &self,
        options: WorkQueueReceiveOptions,
    ) -> Result<Option<Box<dyn WorkQueueDelivery>>, WorkQueueError> {
        if options.lease_duration().is_some() {
            return Err(WorkQueueError::unsupported(
                "memory queue realization did not negotiate `configurable-lease-duration`",
            ));
        }

        let now = Instant::now();
        let lease_until = now
            .checked_add(self.default_lease_duration)
            .ok_or_else(|| {
                WorkQueueError::invalid_request(
                    "default delivery lease exceeds the supported time range",
                )
            })?;
        let mut state = self.lock_state()?;
        let Some(index) = state
            .messages
            .iter()
            .position(|message| message.is_available(now))
        else {
            return Ok(None);
        };
        let message = state
            .messages
            .get_mut(index)
            .ok_or_else(|| WorkQueueError::internal("selected queue message disappeared"))?;
        message.generation = message
            .generation
            .checked_add(1)
            .ok_or_else(|| WorkQueueError::internal("queue delivery generation overflow"))?;
        message.lease_until = Some(lease_until);

        Ok(Some(Box::new(InMemoryDelivery {
            state: Arc::clone(&self.state),
            message_id: message.id,
            generation: message.generation,
            body: message.body.clone(),
        })))
    }
}

#[derive(Debug)]
struct InMemoryDelivery {
    state: Arc<Mutex<QueueState>>,
    message_id: u64,
    generation: u64,
    body: Vec<u8>,
}

impl InMemoryDelivery {
    fn lock_state(&self) -> Result<MutexGuard<'_, QueueState>, WorkQueueError> {
        self.state
            .lock()
            .map_err(|_| WorkQueueError::internal("in-memory queue lock was poisoned"))
    }

    fn message_index(state: &QueueState, message_id: u64) -> Option<usize> {
        state
            .messages
            .iter()
            .position(|message| message.id == message_id)
    }

    fn require_current_generation(
        state: &QueueState,
        index: usize,
        generation: u64,
    ) -> Result<(), WorkQueueError> {
        let message = state
            .messages
            .get(index)
            .ok_or_else(|| WorkQueueError::invalid_delivery("queue delivery no longer exists"))?;
        if message.generation != generation {
            return Err(WorkQueueError::invalid_delivery(
                "queue delivery was superseded by a newer receive attempt",
            ));
        }
        Ok(())
    }
}

impl WorkQueueDelivery for InMemoryDelivery {
    fn body(&self) -> &[u8] {
        &self.body
    }

    fn ack(self: Box<Self>) -> Result<(), WorkQueueError> {
        let mut state = self.lock_state()?;
        let index = Self::message_index(&state, self.message_id)
            .ok_or_else(|| WorkQueueError::invalid_delivery("queue delivery no longer exists"))?;
        Self::require_current_generation(&state, index, self.generation)?;
        state.messages.remove(index);
        Ok(())
    }

    fn retry(self: Box<Self>, options: WorkQueueRetryOptions) -> Result<(), WorkQueueError> {
        if options.delay().is_some() {
            return Err(WorkQueueError::unsupported(
                "memory queue realization did not negotiate `delayed-retry`",
            ));
        }

        let mut state = self.lock_state()?;
        let index = Self::message_index(&state, self.message_id)
            .ok_or_else(|| WorkQueueError::invalid_delivery("queue delivery no longer exists"))?;
        Self::require_current_generation(&state, index, self.generation)?;
        let message = state
            .messages
            .get_mut(index)
            .ok_or_else(|| WorkQueueError::invalid_delivery("queue delivery no longer exists"))?;
        message.lease_until = None;
        Ok(())
    }
}

/// Provider-realization adapter for the local in-memory work queue.
#[derive(Debug, Default, Clone, Copy)]
pub struct InMemoryWorkQueueAdapter;

impl presolve_provider_realization::ProviderAdapter for InMemoryWorkQueueAdapter {
    fn realize(
        &self,
        profile: &presolve_provider_realization::ProviderRealizationProfile,
        requested: &[presolve_provider_realization::ProviderCapabilityRequest],
        _secrets: &dyn presolve_provider_realization::SecretResolver,
    ) -> Result<
        presolve_provider_realization::RealizedProvider,
        presolve_provider_realization::ProviderAdapterError,
    > {
        use presolve_provider_realization::ProviderAdapterError;
        use presolve_provider_sdk::WORK_QUEUE_INTERFACE;
        use semver::Version;

        let supported_version = Version::new(0, 1, 0);

        for capability in requested {
            if capability.interface() != WORK_QUEUE_INTERFACE {
                return Err(ProviderAdapterError::UnsupportedCapability {
                    interface: capability.interface().to_owned(),
                });
            }

            if capability.version() != &supported_version {
                return Err(ProviderAdapterError::UnsupportedVersion {
                    interface: WORK_QUEUE_INTERFACE.to_owned(),
                    selected_version: capability.version().clone(),
                    supported_version: supported_version.clone(),
                });
            }

            if !capability.negotiated_features().is_empty() {
                return Err(ProviderAdapterError::UnsupportedFeatures {
                    interface: WORK_QUEUE_INTERFACE.to_owned(),
                    features: capability.negotiated_features().join(", "),
                });
            }
        }

        Ok(
            presolve_provider_realization::RealizedProvider::new(*profile.provider_id())
                .with_work_queue(Arc::new(InMemoryWorkQueueProvider::new())),
        )
    }
}

#[cfg(test)]
mod tests {
    use presolve_provider_conformance::{
        WorkQueueConformanceOptions, assert_work_queue_conformance,
    };
    use presolve_provider_realization::{
        EnvironmentSecretResolver, ProviderAdapter, ProviderAdapterId, ProviderCapabilityRequest,
        ProviderRealizationProfile,
    };
    use presolve_provider_sdk::{
        WORK_QUEUE_INTERFACE, WorkQueueErrorCode, WorkQueueProvider, WorkQueueReceiveOptions,
        WorkQueueRetryOptions, WorkQueueSendOptions,
    };
    use semver::Version;

    use super::*;

    #[test]
    fn baseline_conformance_passes_without_wall_clock_waits() {
        let provider = InMemoryWorkQueueProvider::with_default_lease(Duration::ZERO);
        assert_work_queue_conformance(
            &provider,
            WorkQueueConformanceOptions::new(Duration::from_secs(1), Duration::from_millis(1)),
        );
    }

    #[test]
    fn active_lease_hides_message_from_another_receive() {
        let provider = InMemoryWorkQueueProvider::with_default_lease(Duration::from_secs(60));
        provider
            .send(b"leased", WorkQueueSendOptions::default())
            .expect("send should succeed");
        let delivery = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("receive should succeed")
            .expect("message should be available");

        assert!(
            provider
                .receive(WorkQueueReceiveOptions::default())
                .expect("second receive should succeed")
                .is_none()
        );
        delivery.ack().expect("ack should succeed");
    }

    #[test]
    fn stale_delivery_cannot_ack_newer_attempt() {
        let provider = InMemoryWorkQueueProvider::with_default_lease(Duration::ZERO);
        provider
            .send(b"generation", WorkQueueSendOptions::default())
            .expect("send should succeed");
        let first = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("first receive should succeed")
            .expect("first delivery should exist");
        let second = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("second receive should succeed")
            .expect("second delivery should exist");

        let error = first
            .ack()
            .expect_err("stale delivery must not settle the newer attempt");
        assert_eq!(error.code(), WorkQueueErrorCode::InvalidDelivery);
        second.ack().expect("current delivery should acknowledge");
    }

    #[test]
    fn baseline_hides_delivery_attempt_metadata() {
        let provider = InMemoryWorkQueueProvider::with_default_lease(Duration::ZERO);
        provider
            .send(b"attempt", WorkQueueSendOptions::default())
            .expect("send should succeed");
        let delivery = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("receive should succeed")
            .expect("delivery should exist");

        assert_eq!(delivery.delivery_attempt(), None);
        delivery.ack().expect("ack should succeed");
    }

    #[test]
    fn baseline_rejects_unnegotiated_optional_parameters() {
        let provider = InMemoryWorkQueueProvider::new();
        let delayed_send = provider
            .send(
                b"delayed",
                WorkQueueSendOptions::new().with_delay(Duration::from_secs(1)),
            )
            .expect_err("delayed send should require negotiation");
        assert_eq!(delayed_send.code(), WorkQueueErrorCode::Unsupported);

        let Err(custom_lease) = provider
            .receive(WorkQueueReceiveOptions::new().with_lease_duration(Duration::from_secs(1)))
        else {
            panic!("custom lease should require negotiation");
        };
        assert_eq!(custom_lease.code(), WorkQueueErrorCode::Unsupported);

        provider
            .send(b"retry", WorkQueueSendOptions::default())
            .expect("baseline send should succeed");
        let delivery = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("baseline receive should succeed")
            .expect("delivery should exist");
        let delayed_retry = delivery
            .retry(WorkQueueRetryOptions::new().with_delay(Duration::from_secs(1)))
            .expect_err("delayed retry should require negotiation");
        assert_eq!(delayed_retry.code(), WorkQueueErrorCode::Unsupported);
    }

    #[test]
    fn adapter_realizes_exact_baseline_contract() {
        let provider_id = presolve_core::ProviderId::new();
        let profile = ProviderRealizationProfile::new(
            provider_id,
            ProviderAdapterId::new(MEMORY_QUEUE_ADAPTER_ID).expect("adapter ID should be valid"),
        );
        let requested = [ProviderCapabilityRequest::new(
            WORK_QUEUE_INTERFACE,
            Version::new(0, 1, 0),
            std::iter::empty::<String>(),
        )];

        let realized = InMemoryWorkQueueAdapter
            .realize(&profile, &requested, &EnvironmentSecretResolver)
            .expect("baseline queue should realize");

        assert_eq!(*realized.provider_id(), provider_id);
        assert!(realized.work_queue().is_some());
    }

    #[test]
    fn adapter_rejects_unproven_optional_features() {
        let provider_id = presolve_core::ProviderId::new();
        let profile = ProviderRealizationProfile::new(
            provider_id,
            ProviderAdapterId::new(MEMORY_QUEUE_ADAPTER_ID).expect("adapter ID should be valid"),
        );
        let requested = [ProviderCapabilityRequest::new(
            WORK_QUEUE_INTERFACE,
            Version::new(0, 1, 0),
            ["delayed-send"],
        )];

        let error = InMemoryWorkQueueAdapter
            .realize(&profile, &requested, &EnvironmentSecretResolver)
            .expect_err("unproven optional features should be rejected");

        assert!(matches!(
            error,
            presolve_provider_realization::ProviderAdapterError::UnsupportedFeatures { .. }
        ));
    }
}
