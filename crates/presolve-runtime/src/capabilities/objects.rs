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

        let info = map_info(read.info());
        let handle = self.allocate_object_handle().ok_or_else(|| {
            store::Error::Internal("object transport handle space exhausted".to_owned())
        })?;

        self.insert_object_read(handle, read);

        Ok(Some(store::ReadSession { handle, info }))
    }

    fn read(&mut self, handle: u64, max_bytes: u64) -> Result<Vec<u8>, store::Error> {
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
        let write = self.object_write_mut(handle).ok_or_else(|| {
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

        write.body.extend_from_slice(&chunk);
        Ok(())
    }

    fn finish_put(&mut self, handle: u64) -> Result<store::ObjectInfo, store::Error> {
        let write = self.remove_object_write(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object write handle {handle}"))
        })?;

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
        self.remove_object_write(handle).ok_or_else(|| {
            store::Error::InvalidHandle(format!("unknown object write handle {handle}"))
        })?;

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
