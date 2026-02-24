use std::path::PathBuf;

use super::prelude::*;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Credentials {
    client_id: String,
    client_secret: String,
}

impl Credentials {
    pub fn new(config_path: Option<PathBuf>) -> Result<Self> {
        // obtain config path
        let config_path = config_path.map(Ok::<_, anyhow::Error>).unwrap_or_else(|| {
            let mut home = std::env::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to obtain home directory."))?;
            home.push(format!("{}-stf.json", env!("CARGO_PKG_NAME")));
            Ok(home)
        })?;

        // open the config file
        let rdr = std::fs::File::open(&config_path).map_err(|e| {
            if let std::io::ErrorKind::NotFound = e.kind() {
                anyhow!("Failed to open '{}'.", config_path.display())
            } else {
                e.into()
            }
        })?;
        // read and deserialize from file
        serde_json::from_reader::<_, Self>(rdr).map_err(Into::into)
    }

    pub const fn client_id(&self) -> &str {
        self.client_id.as_str()
    }

    pub const fn client_secret(&self) -> &str {
        self.client_secret.as_str()
    }
}
