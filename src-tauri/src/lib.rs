#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// ───────────────────────── Imports ─────────────────────────
use tauri_plugin_http::reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::TcpStream;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State, Window};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
}; // HTTP client
   // ───────────────────────── Constants ─────────────────────────
const SIDECAR_NAME: &str = "candle-vllm";
const DEFAULT_PORT: u16 = 1234;
const MAX_HISTORY_MSGS: usize = 100;

/// System prompt that drives ZarSage’s voice
const PREAMBLE: &str = r#"
You are Message Mate, a professional and efficient AI writing assistant dedicated to helping users craft clear, precise, and effective communications.

• At the start of each new interaction, briefly and professionally introduce yourself, greet the user warmly, and prompt them to provide details about their message request.
• After your initial introduction, avoid repeating your name or full introduction unless necessary for clarity.
• Maintain a professional yet approachable tone, ensuring clarity and effectiveness in all communications.
• Provide concise, actionable guidance for writing emails, reports, business correspondence, formal letters, announcements, and social media posts.
• Tailor your responses based on the user's specified context, audience, tone, and style; politely request additional information if details are insufficient.
• Use clear, professional language suitable for diverse users; simplify complex information and offer relevant examples or templates when helpful.
• Be transparent if you're uncertain about any details, and suggest logical next steps or request further clarification.
• Conclude each interaction positively, reinforcing your availability for further assistance.
"#;


// ───────────────────────── State ─────────────────────────
struct LlmServerState {
    child: Arc<Mutex<Option<CommandChild>>>,
    start_attempted: Arc<Mutex<bool>>,
    port: Arc<Mutex<u16>>,
    histories: Arc<Mutex<HashMap<String, Vec<Value>>>>,
}

impl LlmServerState {
    fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            start_attempted: Arc::new(Mutex::new(false)),
            port: Arc::new(Mutex::new(DEFAULT_PORT)),
            histories: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// ───────────────────────── Helpers ─────────────────────────
fn is_port_available(port: u16) -> bool {
    TcpStream::connect(("127.0.0.1", port)).is_err()
}

fn find_available_port(start_port: u16) -> u16 {
    (start_port..start_port + 10)
        .find(|p| is_port_available(*p))
        .unwrap_or(start_port)
}

fn kill_process_on_port(port: u16) -> Result<(), String> {
    let output = Command::new("lsof")
        .args(["-i", &format!(":{port}"), "-t"])
        .output()
        .map_err(|e| format!("Failed to execute lsof: {e}"))?;

    let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if pid.is_empty() {
        return Err(format!("No process found using port {port}"));
    }

    Command::new("kill")
        .args(["-9", &pid])
        .status()
        .map_err(|e| format!("Failed to kill process {pid}: {e}"))?;

    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

// ───────────────────────── Sidecar management ─────────────────────────
#[tauri::command]
async fn start_llm(app: AppHandle, state: State<'_, LlmServerState>) -> Result<(), String> {
    if state.child.lock().unwrap().is_some() {
        println!("[Rust] LLM already running.");
        return Ok(());
    }

    *state.start_attempted.lock().unwrap() = true;

    // choose port
    let mut port = DEFAULT_PORT;
    if !is_port_available(port) {
        println!("[Rust] Default port {port} busy, trying to free…");
        if kill_process_on_port(port).is_err() {
            port = find_available_port(port + 1);
        }
    }
    *state.port.lock().unwrap() = port;
    println!("[Rust] Using port {port}");

    // spawn sidecar
    let port_str = port.to_string(); // avoid temporary lifetime

    let args = vec![
        "--port".to_string(),
        port_str,
        "--model-id".to_string(),
        "unsloth/Qwen3-4B-GGUF".to_string(),
        "--weight-file".to_string(),
        "Qwen3-4B-Q4_0.gguf".to_string(),
        "qwen3".to_string(),
        "--quant".to_string(),
        "gguf".to_string(),
        "--temperature".to_string(),
        "0.0".to_string(),
        "--penalty".to_string(),
        "1.0".to_string(),
    ];

    let (mut rx, child) = app
        .shell()
        .sidecar(SIDECAR_NAME)
        .map_err(|e| format!("Sidecar not found: {e}"))?
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {e}"))?;

    println!("[Rust] Sidecar PID: {:?}", child.pid());
    *state.child.lock().unwrap() = Some(child);

    let app_handle = app.clone();
    let state_child = state.child.clone();
    let state_attempted = state.start_attempted.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(l) => {
                    app_handle
                        .emit("sidecar-stdout", String::from_utf8_lossy(&l).to_string())
                        .ok();
                }
                CommandEvent::Stderr(l) => {
                    app_handle
                        .emit("sidecar-stderr", String::from_utf8_lossy(&l).to_string())
                        .ok();
                }
                CommandEvent::Error(m) => {
                    app_handle.emit("sidecar-error", m).ok();
                }
                CommandEvent::Terminated(_) => {
                    *state_child.lock().unwrap() = None;
                    *state_attempted.lock().unwrap() = false;
                    break;
                }
                _ => {}
            }
        }
    });

    // give the server a moment to come up
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok(())
}

// ───────────────────────── Chat helpers ─────────────────────────
fn get_history<'a>(
    histories: &'a mut HashMap<String, Vec<Value>>,
    label: &str,
) -> &'a mut Vec<Value> {
    histories.entry(label.to_string()).or_insert_with(Vec::new)
}

fn trim_history(history: &mut Vec<Value>) {
    if history.len() > MAX_HISTORY_MSGS {
        let drop = history.len() - MAX_HISTORY_MSGS;
        history.drain(0..drop);
    }
}

// ───────────────────────── Commands ─────────────────────────
#[tauri::command]
async fn ask_qwen(
    prompt: String,
    window: Window,
    state: State<'_, LlmServerState>,
) -> Result<String, String> {
    let label = window.label().to_string();
    let port = *state.port.lock().unwrap();

    // build / extend history
    let payload_messages = {
        let mut histories = state.histories.lock().unwrap();
        let history = get_history(&mut histories, &label);
        if history.is_empty() {
            history.push(json!({ "role": "system", "content": PREAMBLE }));
        }
        history.push(json!({ "role": "user", "content": &prompt }));
        trim_history(history);
        history.clone() // clone so the lock is dropped before await
    };

    // call server
    let client = Client::new();
    let res = client
        .post(format!("http://127.0.0.1:{port}/v1/chat/completions"))
        .json(&json!({ "model": "qwen3", "messages": payload_messages }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("Server error {}", res.status()));
    }

    let body: Value = res.json().await.map_err(|e| e.to_string())?;
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Bad response")?
        .to_owned();

    // push assistant reply
    {
        let mut histories = state.histories.lock().unwrap();
        let history = get_history(&mut histories, &label);
        history.push(json!({ "role": "assistant", "content": &content }));
        trim_history(history);
    }

    Ok(content)
}

// ───────────────────────── App Entrypoint ─────────────────────────
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .manage(LlmServerState::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_llm,
            ask_qwen
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                let state: State<'_, LlmServerState> = window.app_handle().state();
                if let Some(child) = state.child.lock().unwrap().take() {
                    let _ = child.kill();
                }
                state.histories.lock().unwrap().remove(window.label());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
