use macaddr::MacAddr6;

pub type Result<T> = core::result::Result<T, Error>;

pub trait IptimeWolFactory {
    type Client: IptimeWolClient;

    async fn login(base_url: &reqwest::Url, id: &str, password: &str) -> Result<Self::Client>;
}

pub trait IptimeWolClient {
    async fn list_pc(&mut self) -> Result<Vec<PcInfo>>;
    async fn send_wol_packet(&mut self, target: &MacAddr6) -> Result<()>;
    async fn logout(self) -> Result<()>;
}

pub struct PcInfo {
    pub name: String,
    pub mac_address: MacAddr6,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Init error")]
    Init,
    #[error("Parsing error")]
    Parsing,
    #[error("Auth error")]
    Auth,
    #[error("Server error")]
    Server,
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}
