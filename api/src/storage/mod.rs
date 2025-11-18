pub mod traits;
pub mod local;
pub mod s3;
pub mod factory;

pub use traits::{FileStorage, StorageType, UploadedFile};
pub use local::LocalStorage;
pub use s3::S3Storage;
pub use factory::StorageFactory;
