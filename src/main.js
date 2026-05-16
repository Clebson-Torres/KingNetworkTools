import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

const hostInput = document.getElementById("host-input");
const dnsInput = document.getElementById("dns-input");
const outputContent = document.getElementById("output-content");
const outputSection = document.getElementById("output");
const statusBar = document.getElementById("status-bar");
const statusText = document.getElementById("status-text");

function showStatus(msg) {
  statusBar.classList.remove("hidden");
  statusText.textContent = msg;
}

function hideStatus() {
  statusBar.classList.add("hidden");
}

function showOutput() {
  outputSection.classList.remove("hidden");
}

function clearOutput() {
  outputContent.textContent = "";
  outputSection.classList.add("hidden");
}

function appendOutput(text) {
  outputContent.textContent += text + "\n";
  outputContent.scrollTop = outputContent.scrollHeight;
}

function appendSection(title) {
  appendOutput("");
  const line = "─".repeat(Math.min(50, title.length + 10));
  appendOutput(line);
  appendOutput("  " + title);
  appendOutput(line);
}

async function withLoading(msg, fn) {
  showStatus(msg);
  showOutput();
  try {
    const result = await fn();
    return result;
  } finally {
    hideStatus();
  }
}

function catchError(err) {
  appendOutput("[ERRO] " + err);
  console.error(err);
}

async function showLocalIp() {
  await withLoading("Obtendo IP local...", async () => {
    appendSection("IP Local");
    const interfaces = await invoke("get_local_ip");
    for (const iface of interfaces) {
      appendOutput("  Interface:  " + iface.name);
      appendOutput("  IP:         " + iface.ip);
      appendOutput("  Gateway:    " + (iface.gateway || "(não detectado)"));
      appendOutput("");
    }
  }).catch(catchError);
}

async function showPublicIp() {
  await withLoading("Consultando IP público...", async () => {
    appendSection("IP Público");
    const ip = await invoke("get_public_ip");
    appendOutput("  IP Público: " + ip);
  }).catch(catchError);
}

async function showDnsLookup() {
  const host = dnsInput.value.trim();
  if (!host) return;
  await withLoading("Resolvendo DNS para " + host + "...", async () => {
    appendSection("DNS Lookup: " + host);
    const result = await invoke("dns_lookup", { host });
    appendOutput("  Host:     " + result.host);
    appendOutput("  IPs:");
    for (const ip of result.addresses) {
      appendOutput("    - " + ip);
    }
    if (result.reverse) {
      appendOutput("  Reverso:  " + result.reverse);
    } else {
      appendOutput("  Reverso:  (não disponível)");
    }
  }).catch(catchError);
}

async function showPing() {
  const host = hostInput.value.trim();
  if (!host) return;
  await withLoading("Executando ping para " + host + "...", async () => {
    appendSection("Ping: " + host);
    const result = await invoke("ping", { host });
    appendOutput("  Host:        " + result.host);
    appendOutput("  Transmitido: " + result.transmitted);
    appendOutput("  Recebido:    " + result.received);
    appendOutput("  Perda:       " + result.loss_pct.toFixed(0) + "%");
    if (result.received > 0) {
      appendOutput("  Mínimo:      " + result.min_ms.toFixed(1) + " ms");
      appendOutput("  Médio:       " + result.avg_ms.toFixed(1) + " ms");
      appendOutput("  Máximo:      " + result.max_ms.toFixed(1) + " ms");
    }
  }).catch(catchError);
}

async function showTraceroute() {
  const host = hostInput.value.trim();
  if (!host) return;
  await withLoading("Traçando rota para " + host + "...", async () => {
    appendSection("Rota até " + host);
    const hops = await invoke("trace_route", { host });
    for (const hop of hops) {
      const num = String(hop.hop_number).padStart(2, " ");
      const addr = hop.address.padEnd(22);
      appendOutput("  " + num + ". " + addr + " (" + hop.latency_ms + ")");
    }
  }).catch(catchError);
}

async function showListeningPorts() {
  await withLoading("Obtendo portas em escuta...", async () => {
    appendSection("Portas em Escuta");
    const ports = await invoke("get_listening_ports");
    if (ports.length === 0) {
      appendOutput("  Nenhuma porta em escuta encontrada.");
    } else {
      for (const p of ports) {
        appendOutput(
          "  " + String(p.port).padStart(5, " ") + "/" + p.protocol.padEnd(4) + " " + p.state
        );
      }
      appendOutput("\n  Total: " + ports.length + " portas");
    }
  }).catch(catchError);
}

async function showPortScan() {
  const host = hostInput.value.trim() || "127.0.0.1";
  await withLoading("Escaneando portas em " + host + "...", async () => {
    appendSection("Scan de Portas: " + host);
    const commonPorts = [
      21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995,
      1433, 1521, 2049, 3306, 3389, 5432, 5900, 5985, 5986, 6379, 8080, 8443,
      9090, 27017,
    ];
    const results = await invoke("scan_ports", {
      host,
      portsList: commonPorts,
    });
    const open = results.filter((r) => r.state === "ABERTA");
    if (open.length === 0) {
      appendOutput("  Nenhuma porta aberta encontrada.");
    } else {
      for (const r of open) {
        appendOutput(
          "  " + String(r.port).padStart(5, " ") + "/TCP  " +
          r.service.padEnd(12) + " " + r.state + "  " + r.latency_ms + "ms"
        );
      }
    }
    appendOutput("\n  Escaneadas: " + results.length + ", Abertas: " + open.length);
  }).catch(catchError);
}

async function showFullReport() {
  await withLoading("Gerando relatório completo...", async () => {
    const host = hostInput.value.trim() || "8.8.8.8";

    appendSection("IP Local");
    try {
      const interfaces = await invoke("get_local_ip");
      for (const iface of interfaces) {
        appendOutput("  Interface: " + iface.name);
        appendOutput("  IP:        " + iface.ip);
        if (iface.gateway) appendOutput("  Gateway:   " + iface.gateway);
      }
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendSection("IP Público");
    try {
      const publicIp = await invoke("get_public_ip");
      appendOutput("  IP Público: " + publicIp);
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendSection("DNS Lookup: " + host);
    try {
      const dnsResult = await invoke("dns_lookup", { host });
      appendOutput("  Host: " + dnsResult.host);
      for (const ip of dnsResult.addresses) {
        appendOutput("  IP:  " + ip);
      }
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendSection("Ping: " + host);
    try {
      const ping = await invoke("ping", { host });
      appendOutput("  Transmitido: " + ping.transmitted);
      appendOutput("  Recebido:    " + ping.received);
      appendOutput("  Perda:       " + ping.loss_pct.toFixed(0) + "%");
      if (ping.received > 0) {
        appendOutput("  Mínimo:      " + ping.min_ms.toFixed(1) + " ms");
        appendOutput("  Médio:       " + ping.avg_ms.toFixed(1) + " ms");
        appendOutput("  Máximo:      " + ping.max_ms.toFixed(1) + " ms");
      }
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendSection("Rota até " + host);
    try {
      const hops = await invoke("trace_route", { host });
      for (const hop of hops) {
        appendOutput(
          "  " + String(hop.hop_number).padStart(2, " ") + ". " +
          hop.address.padEnd(22) + " (" + hop.latency_ms + ")"
        );
      }
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendSection("Portas em Escuta");
    try {
      const ports = await invoke("get_listening_ports");
      if (ports.length === 0) {
        appendOutput("  Nenhuma porta em escuta encontrada.");
      } else {
        for (const p of ports) {
          appendOutput("  " + String(p.port).padStart(5, " ") + "/" + p.protocol.padEnd(4) + " " + p.state);
        }
      }
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendSection("Scan de Portas: " + host);
    try {
      const commonPorts = [
        21, 22, 23, 25, 53, 80, 110, 111, 135, 139, 143, 443, 445, 993, 995,
        1433, 1521, 2049, 3306, 3389, 5432, 5900, 5985, 5986, 6379, 8080, 8443,
        9090, 27017,
      ];
      const results = await invoke("scan_ports", { host, portsList: commonPorts });
      const open = results.filter((r) => r.state === "ABERTA");
      if (open.length === 0) {
        appendOutput("  Nenhuma porta aberta encontrada.");
      } else {
        for (const r of open) {
          appendOutput(
            "  " + String(r.port).padStart(5, " ") + "/TCP  " +
            r.service.padEnd(12) + " " + r.state + "  " + r.latency_ms + "ms"
          );
        }
      }
      appendOutput("\n  Escaneadas: " + results.length + ", Abertas: " + open.length);
    } catch (e) {
      appendOutput("  [ERRO] " + e);
    }

    appendOutput("");
    appendOutput("═".repeat(50));
    appendOutput("  RELATÓRIO CONCLUÍDO");
    appendOutput("═".repeat(50));
  }).catch(catchError);
}

function copyOutput() {
  const text = outputContent.textContent;
  navigator.clipboard.writeText(text).then(
    () => {
      const btn = document.getElementById("btn-copy");
      btn.textContent = "✅ Copiado!";
      setTimeout(() => (btn.textContent = "📋 Copiar"), 2000);
    },
    () => {
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
  );
}

async function exportReport() {
  const text = outputContent.textContent;
  if (!text.trim()) return;

  const date = new Date().toISOString().slice(0, 19).replace(/[T:-]/g, "");
  const defaultName = `relatorio_rede_${date}.txt`;

  try {
    const path = await save({
      defaultPath: defaultName,
      filters: [{ name: "Texto", extensions: ["txt"] }],
    });

    if (!path) return;

    await writeTextFile(path, text);
    appendOutput(`\nRelatório exportado: ${path}`);
  } catch (err) {
    appendOutput(`\n[ERRO] Falha ao exportar: ${err}`);
  }
}

document.addEventListener("DOMContentLoaded", () => {
  document.getElementById("btn-local-ip").addEventListener("click", showLocalIp);
  document.getElementById("btn-public-ip").addEventListener("click", showPublicIp);
  document.getElementById("btn-dns").addEventListener("click", showDnsLookup);
  document.getElementById("btn-ping").addEventListener("click", showPing);
  document.getElementById("btn-traceroute").addEventListener("click", showTraceroute);
  document.getElementById("btn-listening").addEventListener("click", showListeningPorts);
  document.getElementById("btn-scan").addEventListener("click", showPortScan);
  document.getElementById("btn-report").addEventListener("click", showFullReport);
  document.getElementById("btn-copy").addEventListener("click", copyOutput);
  document.getElementById("btn-export").addEventListener("click", exportReport);
  document.getElementById("btn-clear").addEventListener("click", clearOutput);

  hostInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") showPing();
  });
  dnsInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter") showDnsLookup();
  });
});
