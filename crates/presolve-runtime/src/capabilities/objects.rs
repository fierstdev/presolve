use std::io::Cursor;

use presolve_provider_sdk::{ObjectInfo, ObjectStoreError, ObjectStoreErrorCode};

use crate::{
    bindings::objects::presolve::objects::store,
    state::{PendingObjectWrite, RuntimeState},
};

const MAX_CHUNK_BYTES: u64 = 1024 * 1024;

impl store::Host for RuntimeState {
    fn get(&mut self, key: String) -> Result<Option<store::ReadSession>, store::Error> {
        let read = self
            .object_store_provider()
            .get(&key)
            .map_err(|error| map_error(&error))?;

        let Some(read) = read else {
            return Ok(None);
        };

        if !self.can_open_object_session() {
            return Err(store::Error::InvalidRequest(
                "too many open object transport sessions".to_owned(),
            ));
        }

        let info = map_info(read.info());
        let handle = self.allocate_object_handle().ok_or_else(|| {
            store::Error::Internal("object transport handle space exhausted".to_owned())
        })?;

        self.insert_object_read(handle, read);

        Ok(Some(store::ReadSession { handle, info }))
    }

    fn read(&mut self, handle: u64, max_bytes: u64) -> Result<Vec<u8>, store::Error> {
        if max_bytes == 0 {
            return Err(store::Error::InvalidRequest(
                "read chunk size must be greater than zero".to_owned(),
            ));
        }

        if max_bytes > MAX_CHUNK_BYTES {
            return Err(store::Error::InvalidRequest(format!(
                "read chunk exceeds maximum of {MAX_CHUNK_BYTES} bytes"
            )));
        }

        let len = usize::try_from(max_bytes)
            .map_err(|_| store::Error::InvalidRequest("read size is too large".to_owned()))?;

        let read = self.object_read_mut(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object read handle {handle}"))
        })?;

        let mut buffer = vec![0_u8; len];
        let count = read
            .body()
            .read(&mut buffer)
            .map_err(|error| store::Error::Internal(format!("object read failed: {error}")))?;

        buffer.truncate(count);
        Ok(buffer)
    }

    fn close_read(&mut self, handle: u64) -> Result<(), store::Error> {
        self.remove_object_read(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object read handle {handle}"))
        })?;

        Ok(())
    }

    fn put(&mut self, key: String, size: u64) -> Result<u64, store::Error> {
        if key.is_empty() {
            return Err(store::Error::InvalidKey(
                "object key must not be empty".to_owned(),
            ));
        }

        let size_usize = usize::try_from(size)
            .map_err(|_| store::Error::InvalidRequest("object size is too large".to_owned()))?;

        if size_usize > self.max_object_buffer_bytes() {
            return Err(store::Error::InvalidRequest(format!(
                "object size exceeds runtime buffer limit of {} bytes",
                self.max_object_buffer_bytes()
            )));
        }

        if !self.can_open_object_session() {
            return Err(store::Error::InvalidRequest(
                "too many open object transport sessions".to_owned(),
            ));
        }

        let handle = self.allocate_object_handle().ok_or_else(|| {
            store::Error::Internal("object transport handle space exhausted".to_owned())
        })?;

        self.insert_object_write(
            handle,
            PendingObjectWrite {
                key,
                size,
                body: Vec::new(),
            },
        );

        Ok(handle)
    }

    fn write(&mut self, handle: u64, chunk: Vec<u8>) -> Result<(), store::Error> {
        let write = self.object_write(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object write handle {handle}"))
        })?;

        let current = u64::try_from(write.body.len())
            .map_err(|_| store::Error::InvalidBody("object body is too large".to_owned()))?;
        let chunk_len = u64::try_from(chunk.len())
            .map_err(|_| store::Error::InvalidBody("object body is too large".to_owned()))?;
        let total = current
            .checked_add(chunk_len)
            .ok_or_else(|| store::Error::InvalidBody("object body is too large".to_owned()))?;

        if total > write.size {
            return Err(store::Error::InvalidBody(format!(
                "object body exceeds declared size of {} bytes",
                write.size
            )));
        }

        if !self.reserve_object_buffer_bytes(chunk.len()) {
            return Err(store::Error::InvalidRequest(format!(
                "pending object writes exceed runtime buffer limit of {} bytes",
                self.max_object_buffer_bytes()
            )));
        }

        let Some(write) = self.object_write_mut(handle) else {
            self.release_object_buffer_bytes(chunk.len());
            return Err(store::Error::InvalidHandle(format!(
                "unknown object write handle {handle}"
            )));
        };

        write.body.extend_from_slice(&chunk);
        Ok(())
    }

    fn finish_put(&mut self, handle: u64) -> Result<store::ObjectInfo, store::Error> {
        let write = self.remove_object_write(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object write handle {handle}"))
        })?;

        self.release_object_buffer_bytes(write.body.len());

        let actual = u64::try_from(write.body.len())
            .map_err(|_| store::Error::InvalidBody("object body is too large".to_owned()))?;

        if actual != write.size {
            return Err(store::Error::InvalidBody(format!(
                "object body length mismatch: expected {} bytes, found {actual}",
                write.size
            )));
        }

        let mut body = Cursor::new(write.body);
        let info = self
            .object_store_provider()
            .put(&write.key, write.size, &mut body)
            .map_err(|error| map_error(&error))?;

        Ok(map_info(&info))
    }

    fn abort_put(&mut self, handle: u64) -> Result<(), store::Error> {
        let write = self.remove_object_write(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object write handle {handle}"))
        })?;

        self.release_object_buffer_bytes(write.body.len());
        Ok(())
    }

    fn head(&mut self, key: String) -> Result<Option<store::ObjectInfo>, store::Error> {
        self.object_store_provider()
            .head(&key)
            .map(|info| info.as_ref().map(map_info))
            .map_err(|error| map_error(&error))
    }

    fn delete(&mut self, key: String) -> Result<bool, store::Error> {
        self.object_store_provider()
            .delete(&key)
            .map_err(|error| map_error(&error))
    }

    fn list_objects(&mut self, prefix: String) -> Result<Vec<store::ObjectInfo>, store::Error> {
        self.object_store_provider()
            .list(&prefix)
            .map(|objects| objects.iter().map(map_info).collect())
            .map_err(|error| map_error(&error))
    }
}

fn map_info(info: &ObjectInfo) -> store::ObjectInfo {
    store::ObjectInfo {
        key: info.key().to_owned(),
        size: info.size(),
    }
}

fn map_error(error: &ObjectStoreError) -> store::Error {
    let message = error.message().to_owned();

    match error.code() {
        ObjectStoreErrorCode::Unavailable => store::Error::Unavailable(message),
        ObjectStoreErrorCode::InvalidKey => store::Error::InvalidKey(message),
        ObjectStoreErrorCode::InvalidBody => store::Error::InvalidBody(message),
        ObjectStoreErrorCode::Internal => store::Error::Internal(message),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use presolve_provider_objects_memory::InMemoryObjectStoreProvider;
    use presolve_provider_sdk::UnavailableKeyValueProvider;

    use super::store;
    use crate::{RuntimeLimits, state::RuntimeState};

    fn state(max_buffer_bytes: usize, max_sessions: usize) -> RuntimeState {
        RuntimeState::new(
            RuntimeLimits::new()
                .with_max_object_buffer_bytes(max_buffer_bytes)
                .with_max_object_sessions(max_sessions),
            Arc::new(UnavailableKeyValueProvider),
            Arc::new(InMemoryObjectStoreProvider::new()),
        )
    }

    #[test]
    fn declared_object_larger_than_transport_buffer_is_rejected() {
        let mut state = state(4, 8);

        let error = <RuntimeState as store::Host>::put(&mut state, "object".to_owned(), 5)
            .expect_err("oversized object declaration should fail");

        assert!(matches!(error, store::Error::InvalidRequest(_)));
    }

    #[test]
    fn aggregate_pending_write_buffer_is_bounded_and_reclaimed() {
        let mut state = state(5, 8);

        let first = <RuntimeState as store::Host>::put(&mut state, "first".to_owned(), 4)
            .expect("first session should open");
        let second = <RuntimeState as store::Host>::put(&mut state, "second".to_owned(), 4)
            .expect("second session should open");

        <RuntimeState as store::Host>::write(&mut state, first, vec![1, 2, 3, 4])
            .expect("first write should fit buffer");

        let error = <RuntimeState as store::Host>::write(&mut state, second, vec![5, 6])
            .expect_err("aggregate host buffer limit should be enforced");
        assert!(matches!(error, store::Error::InvalidRequest(_)));

        <RuntimeState as store::Host>::abort_put(&mut state, first)
            .expect("aborting first session should reclaim buffer");

        <RuntimeState as store::Host>::write(&mut state, second, vec![5, 6])
            .expect("reclaimed buffer should become available");
        <RuntimeState as store::Host>::abort_put(&mut state, second)
            .expect("second session should abort");
    }

    #[test]
    fn open_object_sessions_are_bounded_and_reclaimed() {
        let mut state = state(16, 1);

        let first = <RuntimeState as store::Host>::put(&mut state, "first".to_owned(), 1)
            .expect("first session should open");

        let error = <RuntimeState as store::Host>::put(&mut state, "second".to_owned(), 1)
            .expect_err("second simultaneous session should fail");
        assert!(matches!(error, store::Error::InvalidRequest(_)));

        <RuntimeState as store::Host>::abort_put(&mut state, first)
            .expect("closing first session should reclaim slot");

        let second = <RuntimeState as store::Host>::put(&mut state, "second".to_owned(), 1)
            .expect("session slot should be reusable");
        <RuntimeState as store::Host>::abort_put(&mut state, second)
            .expect("second session should abort");
    }

    #[test]
    fn aborted_write_handle_becomes_stale() {
        let mut state = state(16, 8);

        let handle = <RuntimeState as store::Host>::put(&mut state, "object".to_owned(), 1)
            .expect("session should open");
        <RuntimeState as store::Host>::abort_put(&mut state, handle).expect("session should abort");

        let error = <RuntimeState as store::Host>::write(&mut state, handle, vec![1])
            .expect_err("aborted handle must be stale");

        assert!(matches!(error, store::Error::InvalidHandle(_)));
    }

    #[test]
    fn failed_finish_closes_session_and_reclaims_buffer() {
        let mut state = state(4, 1);

        let handle = <RuntimeState as store::Host>::put(&mut state, "first".to_owned(), 4)
            .expect("session should open");
        <RuntimeState as store::Host>::write(&mut state, handle, vec![1, 2, 3])
            .expect("partial write should succeed");

        let error = <RuntimeState as store::Host>::finish_put(&mut state, handle)
            .expect_err("incomplete object should fail");
        assert!(matches!(error, store::Error::InvalidBody(_)));

        let stale_error = <RuntimeState as store::Host>::abort_put(&mut state, handle)
            .expect_err("failed finish must close the session");
        assert!(matches!(stale_error, store::Error::InvalidHandle(_)));

        let replacement = <RuntimeState as store::Host>::put(&mut state, "second".to_owned(), 4)
            .expect("failed finish should reclaim session and buffer budget");
        <RuntimeState as store::Host>::write(&mut state, replacement, vec![4, 3, 2, 1])
            .expect("reclaimed buffer should accept a complete write");
        <RuntimeState as store::Host>::abort_put(&mut state, replacement)
            .expect("replacement should abort cleanly");
    }

    #[test]
    fn zero_sized_read_request_is_rejected() {
        let mut state = state(16, 8);

        let error = <RuntimeState as store::Host>::read(&mut state, 999, 0)
            .expect_err("zero-sized read is ambiguous with EOF");

        assert!(matches!(error, store::Error::InvalidRequest(_)));
    }
}
