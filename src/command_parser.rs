use macaddr::MacAddr6;

#[derive(clap::Parser)]
#[command(
    name = "iptime-wol-cli-newui",
    about = "Command line tools for iptime wol(new ui)"
)]
pub struct Command {
    #[command(subcommand)]
    pub command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    #[command(about = "lists available pc")]
    List {
        #[arg(short = 'm', long = "url", help = "url of management page")]
        url: reqwest::Url,
        #[arg(short = 'u', long = "username", help = "username of managment page")]
        username: String,
        #[arg(short = 'p', long = "password", help = "password of management page")]
        password: String,
    },
    #[command(about = "sends power on signal to pc")]
    On {
        #[arg(short = 'm', long = "url", help = "url of management page")]
        url: reqwest::Url,
        #[arg(short = 'u', long = "username", help = "username of managment page")]
        username: String,
        #[arg(short = 'p', long = "password", help = "password of management page")]
        password: String,
        #[arg(short = 't', long = "target", help = "target mac address of pc")]
        mac_address: MacAddr6,
    },
}
