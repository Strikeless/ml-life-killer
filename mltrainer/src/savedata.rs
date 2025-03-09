use std::{fs, path::Path};

use libml::network::Network;
use serde::{Deserialize, Serialize};

use crate::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub config: Config,

    #[serde(with = "base64_msgpack")]
    pub network: Network,
}

pub fn save<P>(path: P, save_data: SaveData)
where
    P: AsRef<Path>,
{
    let save_data_serialized =
        serde_json::to_string_pretty(&save_data).expect("Couldn't serialize save data");

    let path = path.as_ref();
    let _ = fs::create_dir_all(path.parent().expect("No parent path"));
    fs::write(path, save_data_serialized).expect("Couldn't write save data");
}

pub fn load<P>(path: P) -> SaveData
where
    P: AsRef<Path>,
{
    let save_data_serialized = fs::read(path).expect("Couldn't read save data");
    serde_json::from_slice(&save_data_serialized).expect("Couldn't deserialize save data")
}

mod base64_msgpack {
    use std::marker::PhantomData;

    use base64::{Engine, engine::GeneralPurposeConfig};
    use serde::{
        Deserializer, Serialize, Serializer,
        de::{DeserializeOwned, Visitor},
    };

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        let value_serialized = rmp_serde::to_vec(value).map_err(serde::ser::Error::custom)?;

        let value_encoded = new_base64_engine().encode(value_serialized);

        serializer.serialize_str(&value_encoded)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        struct Base64MsgpackVisitor<T> {
            _input_phantom: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for Base64MsgpackVisitor<T>
        where
            T: DeserializeOwned,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Base64 encoded and MessagePack serialized data")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value_decoded = new_base64_engine()
                    .decode(value)
                    .map_err(serde::de::Error::custom)?;

                rmp_serde::from_slice(&value_decoded).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Base64MsgpackVisitor {
            _input_phantom: PhantomData,
        })
    }

    fn new_base64_engine() -> impl Engine {
        base64::engine::GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            GeneralPurposeConfig::default(),
        )
    }
}
