#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Version {
    OldUi,
    NewUi,
}

pub async fn detect_version(base_url: &reqwest::Url) -> Option<Version> {
    let client = reqwest::Client::new();
    let status_code = client
        .get(base_url.join("ui").ok()?)
        .send()
        .await
        .ok()?
        .status();

    let version = if status_code.is_success() {
        Version::NewUi
    } else {
        Version::OldUi
    };

    Some(version)
}
