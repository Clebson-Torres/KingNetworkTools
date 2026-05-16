use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub ip: String,
    pub gateway: String,
}

pub fn get_local_ip_info() -> Result<Vec<InterfaceInfo>, String> {
    let mut interfaces = Vec::new();

    let ip = local_ip_address::local_ip().map_err(|e| format!("Erro ao obter IP local: {}", e))?;

    interfaces.push(InterfaceInfo {
        name: "principal".to_string(),
        ip: ip.to_string(),
        gateway: get_default_gateway().unwrap_or_default(),
    });

    Ok(interfaces)
}

fn get_default_gateway() -> Option<String> {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("netstat")
            .args(["-rn"])
            .output()
            .ok()?
    } else {
        std::process::Command::new("ip")
            .args(["route", "show", "default"])
            .output()
            .ok()?
    };

    let stdout = String::from_utf8_lossy(&output.stdout);

    if cfg!(target_os = "windows") {
        for line in stdout.lines() {
            if line.contains("0.0.0.0") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[2] != "0.0.0.0" {
                    return Some(parts[2].to_string());
                }
            }
        }
    } else {
        if let Some(line) = stdout.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                return Some(parts[2].to_string());
            }
        }
    }

    None
}

pub fn get_public_ip_address() -> Result<String, String> {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();
    let agent = ureq::Agent::new_with_config(config);

    let resp = agent
        .get("https://api.ipify.org")
        .call()
        .map_err(|e| format!("Falha ao consultar IP público: {}", e))?;

    let mut body = resp.into_body();
    let ip = body
        .read_to_string()
        .map_err(|e| format!("Erro ao ler resposta: {}", e))?;

    Ok(ip.trim().to_string())
}
