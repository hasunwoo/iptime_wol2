use macaddr::MacAddr6;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use crate::interface::{self, IptimeWolClient, IptimeWolFactory};

pub struct ClientBuilder;

pub struct Client {
    client: reqwest::Client,
    service_url: reqwest::Url,
}

impl IptimeWolFactory for ClientBuilder {
    type Client = Client;

    async fn login(
        base_url: &reqwest::Url,
        id: &str,
        password: &str,
    ) -> interface::Result<Self::Client> {
        let mut client = build_client(base_url)?;
        let service_url = service_url(base_url);
        login(&mut client, &service_url, id, password).await?;
        Ok(Client {
            client,
            service_url,
        })
    }
}

impl IptimeWolClient for Client {
    async fn list_pc(&mut self) -> interface::Result<Vec<interface::PcInfo>> {
        let pc_list = wol_show(&mut self.client, &self.service_url).await?;
        let pc_list = pc_list.into_iter().map(Into::into).collect();
        Ok(pc_list)
    }

    async fn send_wol_packet(&mut self, target: &MacAddr6) -> interface::Result<()> {
        wol_signal(&mut self.client, &self.service_url, target).await
    }

    async fn logout(mut self) -> interface::Result<()> {
        logout(&mut self.client, &self.service_url).await
    }
}

impl From<PcInfo> for interface::PcInfo {
    fn from(value: PcInfo) -> Self {
        Self {
            name: value.pc_name,
            mac_address: value.mac_addr,
        }
    }
}

fn build_client(base_url: &reqwest::Url) -> interface::Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Referer",
        HeaderValue::from_str(&format!("{}ui/wol", base_url.as_str()))
            .map_err(|_| interface::Error::Init)?,
    );
    headers.insert(
        "Origin",
        HeaderValue::from_str(base_url.as_str()).map_err(|_| interface::Error::Init)?,
    );

    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .default_headers(headers)
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0",
        )
        .build()
        .map_err(|_| interface::Error::Init)?;
    Ok(client)
}

fn service_url(base_url: &reqwest::Url) -> reqwest::Url {
    base_url.join("cgi/service.cgi").unwrap()
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
struct ApiResponse<Res = serde_json::Value, Err = serde_json::Value> {
    result: Option<Res>,
    error: Option<Err>,
}

async fn login(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
    id: &str,
    password: &str,
) -> interface::Result<()> {
    let resp = client
        .post(service_url.clone())
        .json(&json!({
            "method": "session/login",
            "params": {
                "id": id,
                "pw": password,
            }
        }))
        .send()
        .await?;

    let json: ApiResponse = resp.json().await?;

    match json.error {
        Some(_) => Err(interface::Error::Server),
        None => Ok(()),
    }
}

async fn logout(client: &mut reqwest::Client, service_url: &reqwest::Url) -> interface::Result<()> {
    let resp = client
        .post(service_url.clone())
        .json(&json!({
            "method": "session/logout",
        }))
        .send()
        .await?;
    let json: ApiResponse = resp.json().await?;

    match json.error {
        Some(_) => Err(interface::Error::Server),
        None => Ok(()),
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize)]
struct PcInfo {
    #[serde(rename = "mac")]
    #[serde(deserialize_with = "crate::serde_util::deserialize_from_str")]
    mac_addr: MacAddr6,
    #[serde(rename = "pcname")]
    pc_name: String,
}

async fn wol_show(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
) -> interface::Result<Vec<PcInfo>> {
    let resp = client
        .post(service_url.clone())
        .json(&json!({
            "method": "wol/show",
        }))
        .send()
        .await?;

    let json: ApiResponse<Vec<PcInfo>> = resp.json().await?;

    if json.error.is_some() {
        return Err(interface::Error::Server);
    }

    match json.result {
        Some(res) => Ok(res),
        None => Err(interface::Error::Server),
    }
}

async fn wol_signal(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
    mac_addr: &MacAddr6,
) -> interface::Result<()> {
    let resp = client
        .post(service_url.clone())
        .json(&json!({
            "method": "wol/signal",
            "params": [mac_addr.to_string()]
        }))
        .send()
        .await?;
    let json: ApiResponse = resp.json().await?;

    match json.error {
        Some(_) => Err(interface::Error::Server),
        None => Ok(()),
    }
}
