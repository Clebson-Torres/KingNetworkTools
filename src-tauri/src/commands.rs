use crate::analyzer::{dns, ip, ports, route, report};

#[tauri::command]
pub async fn get_local_ip() -> Result<Vec<ip::InterfaceInfo>, String> {
    tokio::task::spawn_blocking(ip::get_local_ip_info)
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}

#[tauri::command]
pub async fn get_public_ip() -> Result<String, String> {
    tokio::task::spawn_blocking(ip::get_public_ip_address)
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}

#[tauri::command]
pub async fn ping(host: String) -> Result<route::PingResult, String> {
    tokio::task::spawn_blocking(move || route::ping_host(&host, 4))
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}

#[tauri::command]
pub async fn trace_route(host: String) -> Result<Vec<route::Hop>, String> {
    tokio::task::spawn_blocking(move || route::trace_route(&host))
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}

#[tauri::command]
pub async fn get_listening_ports() -> Result<Vec<ports::ListeningPort>, String> {
    tokio::task::spawn_blocking(ports::get_listening_ports)
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}

#[tauri::command]
pub async fn scan_ports(host: String, ports_list: Vec<u16>) -> Vec<ports::ScanResult> {
    tokio::task::spawn_blocking(move || ports::scan_ports(&host, &ports_list, 1500))
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn dns_lookup(host: String) -> Result<dns::DnsResult, String> {
    tokio::task::spawn_blocking(move || dns::dns_lookup(&host))
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}

#[tauri::command]
pub async fn generate_report() -> Result<String, String> {
    tokio::task::spawn_blocking(report::generate_full_report)
        .await
        .map_err(|e| format!("Erro interno: {}", e))?
}
