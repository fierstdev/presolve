use std::time::Duration;

use presolve_provider_sdk::{
    WorkQueueError, WorkQueueErrorCode, WorkQueueReceiveOptions, WorkQueueRetryOptions,
    WorkQueueSendOptions,
};
use wasmtime::component::Resource;

use crate::{bindings::queue::presolve::queue::work_queue, state::RuntimeState};

impl work_queue::Host for RuntimeState {
    fn send(
        &mut self,
        body: Vec<u8>,
        options: work_queue::SendOptions,
    ) -> Result<(), work_queue::Error> {
        self.work_queue_provider()
            .send(&body, map_send_options(options))
            .map_err(|error| map_error(&error))
    }

    fn receive(
        &mut self,
        options: work_queue::ReceiveOptions,
    ) -> Result<Option<Resource<work_queue::Delivery>>, work_queue::Error> {
        if !self.can_open_queue_delivery() {
            return Err(work_queue::Error::Internal(
                "too many open work-queue deliveries".to_owned(),
            ));
        }

        let Some(delivery) = self
            .work_queue_provider()
            .receive(map_receive_options(options))
            .map_err(|error| map_error(&error))?
        else {
            return Ok(None);
        };

        let handle = self.allocate_queue_handle().ok_or_else(|| {
            work_queue::Error::Internal("work-queue runtime handle space exhausted".to_owned())
        })?;
        self.insert_queue_delivery(handle, delivery);

        Ok(Some(Resource::new_own(handle)))
    }

    fn ack(
        &mut self,
        delivery: Resource<work_queue::Delivery>,
    ) -> Result<(), work_queue::Error> {
        let delivery = self.remove_queue_delivery(delivery.rep()).ok_or_else(|| {
            work_queue::Error::InvalidDelivery("unknown work-queue delivery resource".to_owned())
        })?;

        delivery.ack().map_err(|error| map_error(&error))
    }

    fn retry(
        &mut self,
        delivery: Resource<work_queue::Delivery>,
        options: work_queue::RetryOptions,
    ) -> Result<(), work_queue::Error> {
        let delivery = self.remove_queue_delivery(delivery.rep()).ok_or_else(|| {
            work_queue::Error::InvalidDelivery("unknown work-queue delivery resource".to_owned())
        })?;

        delivery
            .retry(map_retry_options(options))
            .map_err(|error| map_error(&error))
    }
}

impl work_queue::HostDelivery for RuntimeState {
    fn body(&mut self, delivery: Resource<work_queue::Delivery>) -> Vec<u8> {
        self.queue_delivery(delivery.rep())
            .expect("Component resource table and work-queue delivery state must stay synchronized")
            .body()
            .to_vec()
    }

    fn delivery_attempt(&mut self, delivery: Resource<work_queue::Delivery>) -> Option<u32> {
        self.queue_delivery(delivery.rep())
            .expect("Component resource table and work-queue delivery state must stay synchronized")
            .delivery_attempt()
    }

    fn drop(&mut self, delivery: Resource<work_queue::Delivery>) -> wasmtime::Result<()> {
        self.remove_queue_delivery(delivery.rep());
        Ok(())
    }
}

fn map_send_options(options: work_queue::SendOptions) -> WorkQueueSendOptions {
    let mut mapped = WorkQueueSendOptions::new();
    if let Some(delay_ms) = options.delay_ms {
        mapped = mapped.with_delay(Duration::from_millis(delay_ms));
    }
    mapped
}

fn map_receive_options(options: work_queue::ReceiveOptions) -> WorkQueueReceiveOptions {
    let mut mapped = WorkQueueReceiveOptions::new();
    if let Some(lease_ms) = options.lease_ms {
        mapped = mapped.with_lease_duration(Duration::from_millis(lease_ms));
    }
    mapped
}

fn map_retry_options(options: work_queue::RetryOptions) -> WorkQueueRetryOptions {
    let mut mapped = WorkQueueRetryOptions::new();
    if let Some(delay_ms) = options.delay_ms {
        mapped = mapped.with_delay(Duration::from_millis(delay_ms));
    }
    mapped
}

fn map_error(error: &WorkQueueError) -> work_queue::Error {
    let message = error.message().to_owned();

    match error.code() {
        WorkQueueErrorCode::Unavailable => work_queue::Error::Unavailable(message),
        WorkQueueErrorCode::InvalidRequest => work_queue::Error::InvalidRequest(message),
        WorkQueueErrorCode::InvalidDelivery => work_queue::Error::InvalidDelivery(message),
        WorkQueueErrorCode::Unsupported => work_queue::Error::Unsupported(message),
        WorkQueueErrorCode::Transient => work_queue::Error::Transient(message),
        WorkQueueErrorCode::Internal => work_queue::Error::Internal(message),
    }
}
