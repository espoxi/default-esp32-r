use anyhow::bail;
use embedded_svc::storage::{SerDe, StorageImpl};
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use postcard::{from_bytes, to_stdvec};
use serde::{de::DeserializeOwned, Serialize};

pub mod storeable;

const STORAGE_RWBUFFER_SIZE: usize = 1024;
pub type DStore = StorageImpl<STORAGE_RWBUFFER_SIZE, EspDefaultNvs, PostCardSerDe>;

pub struct PostCardSerDe;
impl SerDe for PostCardSerDe {
    type Error = anyhow::Error;

    fn serialize<'a, T>(&self, slice: &'a mut [u8], value: &T) -> Result<&'a [u8], Self::Error>
    where
        T: Serialize,
    {
        match to_stdvec(value) {
            Ok(v) => {
                let len = v.len();
                if len > slice.len() {
                    bail!("Slice too small");
                }
                slice[..len].copy_from_slice(&v);
                Ok(&slice[..len])
            }
            Err(e) => bail!("Serialization error: {}", e),
        }
    }

    fn deserialize<T>(&self, slice: &[u8]) -> Result<T, Self::Error>
    where
        T: DeserializeOwned,
    {
        match from_bytes(slice) {
            Ok(v) => Ok(v),
            Err(e) => bail!("Deserialization error: {}", e),
        }
    }
}

// pub struct Storage {}

// impl Storage {
pub fn default() -> DStore {
    let nvsp = EspDefaultNvsPartition::take().expect("Failed to take NVS partition");
    let rs = EspDefaultNvs::new(nvsp, "breb", true).expect("Failed to create nvs");
    StorageImpl::new(rs, PostCardSerDe)
}
// }
