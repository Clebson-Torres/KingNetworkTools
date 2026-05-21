use serde::Serialize;

const GITHUB_API: &str =
    "https://api.github.com/repos/Clebson-Torres/KingNetworkTools/releases/latest";

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: String,
    pub release_notes: String,
}

fn parse_version(v: &str) -> Vec<u32> {
    let v = v.trim_start_matches('v');
    v.split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    latest != current && parse_version(latest) > parse_version(current)
}

pub fn check_for_update() -> UpdateInfo {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let config = ureq::config::Config::builder()
        .timeout_global(Some(std::time::Duration::from_secs(8)))
        .build();
    let agent = ureq::Agent::new_with_config(config);

    let result: Result<(String, String, String), String> = (|| {
        let resp = agent
            .get(GITHUB_API)
            .header("User-Agent", "KingNetworkTools/1.0")
            .header("Accept", "application/json")
            .call()
            .map_err(|e| format!("{}", e))?;

        let mut body = resp.into_body();
        let text = body
            .read_to_string()
            .map_err(|e| format!("Leitura: {}", e))?;

        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| format!("JSON: {}", e))?;

        let tag = json["tag_name"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches('v')
            .to_string();

        let url = json["html_url"].as_str().unwrap_or("").to_string();
        let notes = json["body"].as_str().unwrap_or("").to_string();

        Ok((tag, url, notes))
    })();

    match result {
        Ok((latest, url, notes)) => {
            let has_update = is_newer(&latest, &current);
            UpdateInfo {
                current_version: current,
                latest_version: latest,
                has_update,
                release_url: url,
                release_notes: notes,
            }
        }
        Err(_) => UpdateInfo {
            current_version: current,
            latest_version: String::new(),
            has_update: false,
            release_url: String::new(),
            release_notes: String::new(),
        },
    }
}

pub fn open_url(url: &str) -> Result<(), String> {
    let opener = if cfg!(target_os = "windows") {
        "cmd"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };

    let args: &[&str] = if cfg!(target_os = "windows") {
        &["/C", "start", "", url]
    } else {
        &[url]
    };

    std::process::Command::new(opener)
        .args(args)
        .output()
        .map_err(|e| format!("Falha ao abrir URL: {}", e))?;

    Ok(())
}
