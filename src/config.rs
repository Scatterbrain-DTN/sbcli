use std::{borrow::Cow, marker::PhantomData, path::PathBuf};

use crate::error::Error;
use crate::error::SbResult as Result;
use dirs::config_dir;
use scatterbrain::crypto::EncodeB64;
use serde::{de::DeserializeOwned, Serialize};

pub struct Config<T, V> {
    pub config: T,
    pub path: PathBuf,
    phantom: PhantomData<V>,
}

impl<'a, T, V> Config<T, V>
where
    T: DeserializeOwned + Serialize + Default + EncodeB64<V>,
    V: DeserializeOwned + Serialize,
{
    pub async fn create_dir_if_exists(&self) -> Result<()> {
        tokio::fs::create_dir_all(
            self.path
                .parent()
                .ok_or_else(|| Error::ConfigDoesNotExist("no parent".to_owned()))?,
        )
        .await?;
        Ok(())
    }

    pub async fn try_persist(&self) -> Result<()> {
        self.create_dir_if_exists().await?;
        if tokio::fs::try_exists(&self.path).await? {
            return Err(Error::ConfigAlreadyExists(
                self.path.to_string_lossy().into_owned(),
            ));
        }
        let t = toml::to_string_pretty(&self.config.b64())?;

        tokio::fs::write(&self.path, t).await?;
        Ok(())
    }

    pub async fn persist(&self) -> Result<()> {
        self.create_dir_if_exists().await?;

        let t = toml::to_string_pretty(&self.config.b64())?;

        tokio::fs::write(&self.path, t).await?;
        Ok(())
    }

    pub async fn try_load_dir(app_name: Cow<'a, str>, mut dir: PathBuf) -> Result<Self> {
        dir.push(app_name.as_ref());
        dir.push("session.toml");

        if !tokio::fs::try_exists(&dir).await? {
            let config = Self {
                config: T::default(),
                path: dir,
                phantom: PhantomData,
            };
            config.try_persist().await?;
            return Ok(config);
        }

        let session: V = toml::from_str(&tokio::fs::read_to_string(&dir).await?)?;

        Ok(Self {
            config: T::from_b64(session)?,
            path: dir,
            phantom: PhantomData,
        })
    }

    pub async fn try_load(app_name: Cow<'a, str>) -> Result<Self> {
        let dir = config_dir().ok_or_else(|| Error::ConfigMissingError)?;
        Self::try_load_dir(app_name, dir).await
    }
}
