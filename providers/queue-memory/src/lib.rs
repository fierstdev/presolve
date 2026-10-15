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

const DEFAULT_LEASE_DURATION: Duration = Duration::from_secs(30);

#[derive(Debug)]
struct Message {
    id: u64,
    body: Vec<u8>,
    available_at: Instant,
    lease_until: Option<Instant>,
    generation: u64,
    delivery_attempts: u32,
}

impl Message {
    fn is_available(&self, now: Instant) -> bool {
        self.available_at <= now && self.lease_until.is_none_or(|deadline| deadline <= now)
    }
}

#[derive(Debug, Default)]
struct QueueState {
    next_id: u64,
    messages: VecDeque<Message>,
}

/// Process-local leased work queue.
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

    /// Creates an empty queue with an explicit baseline delivery lease.
    ///
    /// A zero-duration lease is useful for deterministic semantic tests because
    /// an abandoned delivery becomes eligible on the next receive without a
    /// wall-clock sleep.
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
        let now = Instant::now();
        let delay = options.delay().unwrap_or(Duration::ZERO);
        let available_at = checked_add(now, delay, "send delay")?;
        let mut state = self.lock_state()?;
        let id = state.next_id;
        state.next_id = state
            .next_id
            .checked_add(1)
            .ok_or_else(|| WorkQueueError::internal("in-memory queue message id overflow"))?;
        state.messages.push_back(Message {
            id,
            body: body.to_vec(),
            available_at,
            lease_until: None,
            generation: 0,
            delivery_attempts: 0,
        });
        Ok(())
    }

    fn receive(
        &self,
        options: WorkQueueReceiveOptions,
    ) -> Result<Option<Box<dyn WorkQueueDelivery>>, WorkQueueError> {
        let now = Instant::now();
        let lease_duration = options
            .lease_duration()
            .unwrap_or(self.default_lease_duration);
        let lease_until = checked_add(now, lease_duration, "delivery lease")?;
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
        message.delivery_attempts = message
            .delivery_attempts
            .checked_add(1)
            .ok_or_else(|| WorkQueueError::internal("queue delivery-attempt count overflow"))?;
        message.lease_until = Some(lease_until);

        Ok(Some(Box::new(InMemoryDelivery {
            state: Arc::clone(&self.state),
            message_id: message.id,
            generation: message.generation,
            body: message.body.clone(),
            delivery_attempt: message.delivery_attempts,
        })))
    }
}

#[derive(Debug)]
struct InMemoryDelivery {
    state: Arc<Mutex<QueueState>>,
    message_id: u64,
    generation: u64,
    body: Vec<u8>,
    delivery_attempt: u32,
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

    fn delivery_attempt(&self) -> Option<u32> {
        Some(self.delivery_attempt)
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
        let now = Instant::now();
        let delay = options.delay().unwrap_or(Duration::ZERO);
        let available_at = checked_add(now, delay, "retry delay")?;
        let mut state = self.lock_state()?;
        let index = Self::message_index(&state, self.message_id)
            .ok_or_else(|| WorkQueueError::invalid_delivery("queue delivery no longer exists"))?;
        Self::require_current_generation(&state, index, self.generation)?;
        let message = state
            .messages
            .get_mut(index)
            .ok_or_else(|| WorkQueueError::invalid_delivery("queue delivery no longer exists"))?;
        message.available_at = available_at;
        message.lease_until = None;
        Ok(())
    }
}

fn checked_add(now: Instant, duration: Duration, field: &str) -> Result<Instant, WorkQueueError> {
    now.checked_add(duration).ok_or_else(|| {
        WorkQueueError::invalid_request(format!("{field} exceeds the supported time range"))
    })
}

#[cfg(test)]
mod tests {
    use presolve_provider_conformance::{
        WorkQueueConformanceOptions, assert_work_queue_conformance,
    };
    use presolve_provider_sdk::{
        WorkQueueErrorCode, WorkQueueProvider, WorkQueueReceiveOptions, WorkQueueSendOptions,
    };

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
    fn delivery_attempt_increments_on_redelivery() {
        let provider = InMemoryWorkQueueProvider::with_default_lease(Duration::ZERO);
        provider
            .send(b"attempt", WorkQueueSendOptions::default())
            .expect("send should succeed");
        let first = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("first receive should succeed")
            .expect("first delivery should exist");
        assert_eq!(first.delivery_attempt(), Some(1));
        drop(first);

        let second = provider
            .receive(WorkQueueReceiveOptions::default())
            .expect("second receive should succeed")
            .expect("second delivery should exist");
        assert_eq!(second.delivery_attempt(), Some(2));
        second.ack().expect("ack should succeed");
    }
}
