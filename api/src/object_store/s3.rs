/// S3-compatible object store implementation.
///
/// Phase 1 stub — returns `EmberTroveError::Internal` for every operation.
/// Phase 6 will wire in the `rust-s3` crate.
use async_trait::async_trait;
use bytes::Bytes;
use common::EmberTroveError;

use super::ObjectStore;

pub struct S3ObjectStore {
    bucket: String,
    endpoint: String,
}

impl S3ObjectStore {
    #[must_use]
    pub fn new(bucket: String, endpoint: String) -> Self {
        Self { bucket, endpoint }
    }
}

#[async_trait]
impl ObjectStore for S3ObjectStore {
    async fn put(
        &self,
        _key: &str,
        _data: Bytes,
        _content_type: &str,
    ) -> Result<(), EmberTroveError> {
        let _ = (&self.bucket, &self.endpoint);
        Err(EmberTroveError::Internal(
            "S3 object store not yet implemented".to_string(),
        ))
    }

    async fn get(&self, _key: &str) -> Result<Bytes, EmberTroveError> {
        Err(EmberTroveError::Internal(
            "S3 object store not yet implemented".to_string(),
        ))
    }

    async fn delete(&self, _key: &str) -> Result<(), EmberTroveError> {
        Err(EmberTroveError::Internal(
            "S3 object store not yet implemented".to_string(),
        ))
    }

    async fn presigned_url(&self, _key: &str, _expires_secs: u32) -> Result<String, EmberTroveError> {
        Err(EmberTroveError::Internal(
            "S3 object store not yet implemented".to_string(),
        ))
    }
}
