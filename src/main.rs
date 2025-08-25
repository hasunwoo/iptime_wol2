use anyhow::Context;
use clap::Parser;
use command_parser::{Command, Subcommand};
use interface::IptimeWolClient;

use macaddr::MacAddr6;

mod command_parser;
mod serde_util;

mod clients;
mod interface;

#[tokio::main]
async fn main() {
    let cli = Command::parse();

    match cli.command {
        Subcommand::On {
            url,
            username,
            password,
            mac_address,
        } => {
            let result = send_wol_signal(&url, &username, &password, &mac_address).await;

            if let Err(e) = result {
                eprintln!("{e}");
            }
        }
        Subcommand::List {
            url,
            username,
            password,
        } => {
            let result = list_pc(&url, &username, &password).await;

            let pc_list = match result {
                Ok(list) => list,
                Err(e) => {
                    eprintln!("{e}");
                    return;
                }
            };

            println!("name /  mac_address");
            for pc in pc_list {
                println!("{} / {}", pc.name, pc.mac_address);
            }
        }
    }
}

async fn send_wol_signal(
    base_url: &reqwest::Url,
    id: &str,
    password: &str,
    mac_addr: &MacAddr6,
) -> anyhow::Result<()> {
    let mut client = clients::login(base_url, id, password)
        .await
        .with_context(|| anyhow::anyhow!("send_wol_signal(): login error"))?;
    client
        .send_wol_packet(mac_addr)
        .await
        .with_context(|| anyhow::anyhow!("send_wol_signal(): error sending wol packet"))?;
    client
        .logout()
        .await
        .with_context(|| anyhow::anyhow!("send_wol_signal(): error while Logout"))
}

async fn list_pc(
    base_url: &reqwest::Url,
    id: &str,
    password: &str,
) -> anyhow::Result<Vec<interface::PcInfo>> {
    let mut client = clients::login(base_url, id, password)
        .await
        .with_context(|| anyhow::anyhow!("list_pc(): login error"))?;
    let list = client
        .list_pc()
        .await
        .with_context(|| anyhow::anyhow!("list_pc(): error while list PCs"))?;
    client
        .logout()
        .await
        .with_context(|| anyhow::anyhow!("list_pc(): error while Logout"))?;
    Ok(list)
}
