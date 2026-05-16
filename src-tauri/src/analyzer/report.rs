use std::time::{SystemTime, UNIX_EPOCH};

use super::ip;
use super::ports;
use super::route;

fn format_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let year = 1970 + (days as f64 / 365.25) as u64;
    let remaining_days = days as u64 % 365;
    let month_days = [
        31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1;
    let mut day = remaining_days + 1;
    for &md in &month_days {
        if day > md {
            day -= md;
            month += 1;
        } else {
            break;
        }
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

pub fn generate_full_report() -> Result<String, String> {
    let mut report = String::new();

    report.push_str("=== RELATÓRIO DE ANÁLISE DE REDE ===\n");
    report.push_str(&format!("Data: {}\n\n", format_timestamp()));

    report.push_str(&generate_ip_section()?);
    report.push_str("\n");
    report.push_str(&generate_listening_ports_section()?);

    Ok(report)
}

fn generate_ip_section() -> Result<String, String> {
    let mut section = String::new();
    section.push_str("--- IP Local ---\n");

    let interfaces = ip::get_local_ip_info()?;
    for iface in &interfaces {
        section.push_str(&format!("  Interface: {}\n", iface.name));
        section.push_str(&format!("  IP: {}\n", iface.ip));
        if !iface.gateway.is_empty() {
            section.push_str(&format!("  Gateway: {}\n", iface.gateway));
        }
    }

    section.push_str("\n--- IP Público ---\n");
    match ip::get_public_ip_address() {
        Ok(public_ip) => {
            section.push_str(&format!("  IP Público: {}\n", public_ip));
        }
        Err(e) => {
            section.push_str(&format!("  Erro: {}\n", e));
        }
    }

    Ok(section)
}

pub fn generate_ping_section(host: &str) -> Result<String, String> {
    let mut section = String::new();
    section.push_str(&format!("--- Ping: {} ---\n", host));

    match route::ping_host(host, 4) {
        Ok(result) => {
            section.push_str(&format!(
                "  Pacotes: {} enviados, {} recebidos ({:.0}% perda)\n",
                result.transmitted, result.received, result.loss_pct
            ));
            if result.received > 0 {
                section.push_str(&format!(
                    "  Latência: min={:.1}ms, média={:.1}ms, máx={:.1}ms\n",
                    result.min_ms, result.avg_ms, result.max_ms
                ));
            }
        }
        Err(e) => {
            section.push_str(&format!("  Erro: {}\n", e));
        }
    }

    Ok(section)
}

pub fn generate_traceroute_section(host: &str) -> Result<String, String> {
    let mut section = String::new();
    section.push_str(&format!("--- Rota até {} ---\n", host));

    match route::trace_route(host) {
        Ok(hops) => {
            for hop in &hops {
                section.push_str(&format!(
                    "  {}: {} ({}ms)\n",
                    hop.hop_number, hop.address, hop.latency_ms
                ));
            }
        }
        Err(e) => {
            section.push_str(&format!("  Erro: {}\n", e));
        }
    }

    Ok(section)
}

fn generate_listening_ports_section() -> Result<String, String> {
    let mut section = String::new();
    section.push_str("--- Portas em Escuta ---\n");

    match ports::get_listening_ports() {
        Ok(list) => {
            if list.is_empty() {
                section.push_str("  Nenhuma porta em escuta encontrada.\n");
            } else {
                for p in &list {
                    section.push_str(&format!(
                        "  {}/{} ({})\n",
                        p.port, p.protocol, p.state
                    ));
                }
            }
        }
        Err(e) => {
            section.push_str(&format!("  Erro: {}\n", e));
        }
    }

    Ok(section)
}

pub fn generate_port_scan_section(host: &str, port_list: &[u16]) -> String {
    let mut section = String::new();
    section.push_str(&format!("--- Scan de Portas: {} ---\n", host));

    let results = ports::scan_ports(host, port_list, 1500);
    let abertas: Vec<_> = results.iter().filter(|r| r.state == "ABERTA").collect();

    if abertas.is_empty() {
        section.push_str("  Nenhuma porta aberta encontrada.\n");
    } else {
        for r in &abertas {
            section.push_str(&format!(
                "  {}/TCP ({}) - {} - {}ms\n",
                r.port, r.service, r.state, r.latency_ms
            ));
        }
    }

    section.push_str(&format!(
        "\n  Total de {} portas escaneadas, {} abertas.\n",
        results.len(),
        abertas.len()
    ));

    section
}
