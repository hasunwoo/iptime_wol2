use crate::interface::{self, IptimeWolClient, IptimeWolFactory, PcInfo};
use macaddr::MacAddr6;
use thiserror::Error;
use version_detector::{Version, detect_version};

mod new_ui;
mod old_ui;
pub mod version_detector;

pub enum Client {
    OldUi(old_ui::Client),
    NewUi(new_ui::Client),
}

impl From<old_ui::Client> for Client {
    fn from(client: old_ui::Client) -> Self {
        Client::OldUi(client)
    }
}

impl From<new_ui::Client> for Client {
    fn from(client: new_ui::Client) -> Self {
        Client::NewUi(client)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Version")]
    InvalidVersion,
    #[error("Wol Error: {0:?}")]
    WolError(#[from] interface::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

pub async fn login(base_url: &reqwest::Url, id: &str, password: &str) -> Result<Client> {
    let version = match detect_version(base_url).await {
        Some(v) => v,
        None => return Err(Error::InvalidVersion),
    };

    match version {
        Version::OldUi => old_ui::ClientBuilder::login(base_url, id, password)
            .await
            .map_err(Into::into)
            .map(Into::into),
        Version::NewUi => new_ui::ClientBuilder::login(base_url, id, password)
            .await
            .map_err(Into::into)
            .map(Into::into),
    }
}

impl IptimeWolClient for Client {
    async fn list_pc(&mut self) -> interface::Result<Vec<PcInfo>> {
        match self {
            Client::OldUi(client) => client.list_pc().await,
            Client::NewUi(client) => client.list_pc().await,
        }
    }

    async fn send_wol_packet(&mut self, target: &MacAddr6) -> interface::Result<()> {
        match self {
            Client::OldUi(client) => client.send_wol_packet(target).await,
            Client::NewUi(client) => client.send_wol_packet(target).await,
        }
    }

    async fn logout(self) -> interface::Result<()> {
        match self {
            Client::OldUi(client) => client.logout().await,
            Client::NewUi(client) => client.logout().await,
        }
    }
}
