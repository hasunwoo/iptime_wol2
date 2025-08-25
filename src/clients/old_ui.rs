use macaddr::MacAddr6;
use reqwest::{
    StatusCode,
    header::{COOKIE, HeaderMap, HeaderValue},
};

use crate::interface::{self, IptimeWolClient, IptimeWolFactory};

pub struct ClientBuilder;

pub struct Client {
    client: reqwest::Client,
    service_url: reqwest::Url,
    emf_session_id: Option<String>,
}

impl IptimeWolFactory for ClientBuilder {
    type Client = Client;

    async fn login(
        base_url: &reqwest::Url,
        id: &str,
        password: &str,
    ) -> interface::Result<Self::Client> {
        let mut client = build_client(base_url)?;
        let session = login(&mut client, base_url, id, password).await?;
        Ok(Client {
            client,
            service_url: base_url.clone(),
            emf_session_id: Some(session),
        })
    }
}

impl IptimeWolClient for Client {
    async fn list_pc(&mut self) -> interface::Result<Vec<interface::PcInfo>> {
        match self.emf_session_id {
            Some(ref session) => {
                let pc_list = wol_show(&mut self.client, &self.service_url, session).await?;
                let pc_list = pc_list.into_iter().map(Into::into).collect();
                Ok(pc_list)
            }
            None => Err(interface::Error::Auth),
        }
    }

    async fn send_wol_packet(&mut self, target: &MacAddr6) -> interface::Result<()> {
        match self.emf_session_id {
            Some(ref session) => {
                wol_signal(&mut self.client, &self.service_url, session, target).await
            }
            None => Err(interface::Error::Auth),
        }
    }

    async fn logout(mut self) -> interface::Result<()> {
        match self.emf_session_id {
            Some(ref session) => logout(&mut self.client, &self.service_url, session).await,
            None => Err(interface::Error::Auth),
        }
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
        HeaderValue::from_str(base_url.as_str()).map_err(|_| interface::Error::Init)?,
    );

    let client = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:138.0) Gecko/20100101 Firefox/138.0",
        )
        .build()
        .map_err(|_| interface::Error::Init)?;
    Ok(client)
}

async fn login(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
    id: &str,
    password: &str,
) -> interface::Result<String> {
    let url = service_url.join("/sess-bin/login_handler.cgi").unwrap();

    let qs = qstring::QString::new(vec![
        ("username", id),
        ("passwd", password),
        ("act", "session_id"),
    ]);

    let resp = client.post(url).body(qs.to_string()).send().await?;

    match resp.status() {
        StatusCode::OK => {
            let session = resp.text().await?;
            Ok(session)
        }
        StatusCode::BAD_GATEWAY => Err(interface::Error::Auth),
        _ => Err(interface::Error::Server),
    }
}

async fn logout(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
    session: &str,
) -> interface::Result<()> {
    let url = service_url
        .join("/sess-bin/login_session.cgi?logout=1")
        .unwrap();

    client
        .get(url)
        .header(COOKIE, &format!("efm_session_id={session}"))
        .send()
        .await?;

    Ok(())
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct PcInfo {
    mac_addr: MacAddr6,
    pc_name: String,
}

async fn wol_show(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
    session: &str,
) -> interface::Result<Vec<PcInfo>> {
    let url = service_url.join("/sess-bin/info.cgi?act=wol_list").unwrap();

    let resp = client
        .get(url)
        .header(COOKIE, &format!("efm_session_id={session}"))
        .send()
        .await?;

    let text = resp.text().await?;

    let pc_list: Vec<PcInfo> = text
        .lines()
        .map(|line| {
            let mut s = line.split_terminator(';');
            let mac_addr = s.next().ok_or(interface::Error::Parsing)?;
            let mac_addr = mac_addr.parse().map_err(|_| interface::Error::Parsing)?;
            let pc_name = s.next().ok_or(interface::Error::Parsing)?;
            Ok(PcInfo {
                mac_addr,
                pc_name: pc_name.into(),
            })
        })
        .collect::<interface::Result<Vec<_>>>()?;

    Ok(pc_list)
}

async fn wol_signal(
    client: &mut reqwest::Client,
    service_url: &reqwest::Url,
    session: &str,
    mac_addr: &MacAddr6,
) -> interface::Result<()> {
    let mut url = service_url.join("/sess-bin/wol_apply.cgi").unwrap();
    let mac_address = mac_addr.to_string();
    url.set_query(Some(&format!(
        "act=wakeup&mac={}",
        urlencoding::encode(&mac_address)
    )));

    let resp = client
        .get(url)
        .header(COOKIE, &format!("efm_session_id={session}"))
        .send()
        .await?;

    let text = resp.text().await?;

    if text.contains("fail") {
        return Err(interface::Error::Server);
    }

    Ok(())
}
