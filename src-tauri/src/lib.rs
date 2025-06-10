// src/main.rs or src/lib.rs

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rig::completion::{Chat, Message};
use tauri::{AppHandle, Manager, State, Emitter};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt
};
use std::sync::{Arc, Mutex};

mod rig_client;

const SIDECAR_NAME: &str = "candle-vllm";

struct LlmServerState {
    child: Arc<Mutex<Option<CommandChild>>>,
    start_attempted: Arc<Mutex<bool>>,
}

impl LlmServerState {
    fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            start_attempted: Arc::new(Mutex::new(false)),
        }
    }
}

#[tauri::command]
async fn start_llm(
    app: AppHandle,
    state: State<'_, LlmServerState>,
) -> Result<(), String> {
    {
        let child_guard = state.child.lock().unwrap();
        if child_guard.is_some() {
            println!("[Rust] LLM server process already initiated or running.");
            return Ok(());
        }
    }

    {
        let mut start_attempted_guard = state.start_attempted.lock().unwrap();
        if *start_attempted_guard {
            println!("[Rust] Previous LLM server start attempt was made. Will proceed to start again if not running.");
        }
        *start_attempted_guard = true;
    }

    println!("[Rust] Attempting to start LLM server with local model...");
    

    let args = vec![
        "--port", "1234",
        "--model-id", "unsloth/Qwen3-4B-GGUF",
        "--weight-file", "Qwen3-4B-Q4_0.gguf",
        "qwen3",
        "--quant", "gguf",
        "--temperature", "0.0",
        "--penalty", "1.0",
    ];

    println!("[Rust] Sidecar args for local model: {:?}", args);

    let (mut rx, child_process_for_move) = app
        .shell()
        .sidecar(SIDECAR_NAME)
        .map_err(|e| format!("[Rust Error] Failed to find sidecar program '{}': {}", SIDECAR_NAME, e))?
        .args(args)
        .spawn()
        .map_err(|e| {
            {
                let mut sa_guard_on_err = state.start_attempted.lock().unwrap();
                *sa_guard_on_err = false;
            }
            format!("[Rust Error] Failed to spawn sidecar '{}': {}", SIDECAR_NAME, e)
        })?;

    let child_pid = child_process_for_move.pid();
    println!("[Rust] Sidecar process spawned with PID: {:?}", child_pid);

    {
        let mut child_guard_for_store = state.child.lock().unwrap();
        *child_guard_for_store = Some(child_process_for_move);
    }

    println!("[Rust] Updated global LlmServerState with new child PID: {}", child_pid);

    let app_handle_clone = app.clone();
    let state_child_arc = state.child.clone();
    let state_start_attempted_arc = state.start_attempted.clone();

    tauri::async_runtime::spawn(async move {
        println!("[Sidecar Monitor] Started for PID: {:?}.", child_pid);
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let log_line = String::from_utf8_lossy(&line);
                    print!("[Sidecar STDOUT]: {}", log_line);
                    app_handle_clone.emit("sidecar-stdout", log_line.to_string()).unwrap_or_default();
                }
                CommandEvent::Stderr(line) => {
                    let err_line = String::from_utf8_lossy(&line);
                    eprint!("[Sidecar STDERR]: {}", err_line);
                    app_handle_clone.emit("sidecar-stderr", err_line.to_string()).unwrap_or_default();
                }
                CommandEvent::Error(message) => {
                    eprintln!("[Sidecar ERROR]: {}", message);
                    app_handle_clone.emit("sidecar-error", message).unwrap_or_default();
                }
                CommandEvent::Terminated(payload) => {
                    eprintln!("[Sidecar Terminated]: PID {:?} Payload: {:?}", child_pid, payload);
                    app_handle_clone.emit("sidecar-terminated", format!("PID {:?} Payload: {:?}", child_pid, payload)).unwrap_or_default();
                    {
                        let mut child_guard_on_terminate = state_child_arc.lock().unwrap();
                        *child_guard_on_terminate = None;
                    }
                    {
                        let mut start_attempted_guard_on_terminate = state_start_attempted_arc.lock().unwrap();
                        *start_attempted_guard_on_terminate = false;
                    }
                    println!("[Rust] Sidecar process terminated, state cleared for PID: {:?}.", child_pid);
                    break;
                }
                _ => {}
            }
        }
        println!("[Sidecar Monitor] Event stream closed for PID: {:?}.", child_pid);
    });

    println!("[Rust] Waiting for sidecar (local model) to initialize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; // Adjusted sleep, monitor logs
    println!("[Rust] Assumed sidecar (local model) started successfully.");

    Ok(())
}

#[tauri::command]
async fn ask_qwen(prompt: String) -> Result<String, String> {
    println!("[Rust] ask_qwen called with prompt: \"{}\"", prompt);
    let client = rig_client::rig_local();
    // This should match the model_type argument given to candle-vllm-server,
    // which is "qwen3" in our start_llm args.
    let model_id_for_rig = "qwen3";

    println!("[Rust] Using model_id for rig client: {}", model_id_for_rig);
    let qwen_agent = client.agent(model_id_for_rig).build();
    let user_message = Message::user(prompt);

    match qwen_agent.chat(user_message, vec![]).await {
        Ok(response) => {
            println!("[Rust] LLM Response: {}", response);
            Ok(response)
        }
        Err(e) => {
            eprintln!("[Rust Error] Error calling LLM: {:?}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LlmServerState::new())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_llm, greet, ask_qwen])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Destroyed => {
                println!("[Rust] Window closing, attempting to terminate sidecar.");
                let state: State<'_, LlmServerState> = window.app_handle().state();
                let mut child_guard = state.child.lock().unwrap();
                if let Some(mut child_process) = child_guard.take() {
                    println!("[Rust] Terminating sidecar PID: {:?}", child_process.pid());
                    if let Err(e) = child_process.kill() {
                        eprintln!("[Rust Error] Failed to kill sidecar process: {}", e);
                    } else {
                        println!("[Rust] Sidecar process kill signal sent.");
                    }
                } else {
                    println!("[Rust] No active sidecar process to terminate on window close.");
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}