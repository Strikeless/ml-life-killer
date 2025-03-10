use std::{fs, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::network::Network;

use super::NetworkPlayerConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSave {
    pub player_config: NetworkPlayerConfig,

    #[serde(with = "base64_msgpack")]
    pub network: Network,
}

impl NetworkSave {
    pub fn save<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let save_data_serialized =
            serde_json::to_string_pretty(&self).context("Couldn't serialize network save")?;

        let path = path.as_ref();
        let parent_path = path.parent().context("No parent path")?;

        let _ = fs::create_dir_all(parent_path);
        fs::write(path, save_data_serialized).context("Couldn't write network save")?;

        Ok(())
    }

    pub fn load<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let save_serialized = fs::read(path).context("Couldn't read network save")?;
        let save = serde_json::from_slice(&save_serialized)
            .context("Couldn't deserialize network save")?;
        Ok(save)
    }
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
