pub mod internal;
mod key;
#[cfg(test)]
mod str;

use std::{error::Error, fmt::Debug, io, sync::Arc};

use arrow::{array::RecordBatch, datatypes::Schema};
use internal::InternalRecordRef;
pub use key::{Key, KeyRef};
use parquet::arrow::ProjectionMask;
use thiserror::Error;

use crate::{
    inmem::immutable::ArrowArrays,
    serdes::{Decode, Encode},
};

pub trait Record: 'static + Sized + Decode + Debug + Send + Sync {
    type Columns: ArrowArrays<Record = Self>;

    type Key: Key;

    type Ref<'r>: RecordRef<'r, Record = Self> + Copy
    where
        Self: 'r;

    fn key(&self) -> <<Self as Record>::Key as Key>::Ref<'_> {
        self.as_record_ref().key()
    }

    fn primary_key_index() -> usize;

    fn as_record_ref(&self) -> Self::Ref<'_>;

    fn arrow_schema() -> &'static Arc<Schema>;
}

pub trait RecordRef<'r>: Clone + Sized + Copy + Encode + Send + Sync {
    type Record: Record;

    fn key(self) -> <<Self::Record as Record>::Key as Key>::Ref<'r>;

    fn from_record_batch(
        record_batch: &'r RecordBatch,
        offset: usize,
        projection_mask: &'r ProjectionMask,
    ) -> InternalRecordRef<'r, Self>;
}

#[derive(Debug, Error)]
pub enum RecordEncodeError {
    #[error("record's field: {field_name} encode error: {error}")]
    Encode {
        field_name: String,
        error: Box<dyn Error + Send + Sync + 'static>,
    },
    #[error("record io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum RecordDecodeError {
    #[error("record's field: {field_name} decode error: {error}")]
    Decode {
        field_name: String,
        error: Box<dyn Error + Send + Sync + 'static>,
    },
    #[error("record io error: {0}")]
    Io(#[from] io::Error),
}
