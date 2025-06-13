//! A single Rig client that targets the local candle-vLLM server.

use rig::providers::openai;

/// 127.0.0.1:1234 is the default port we start candle-vLLM on.
/// The token can be any non-empty string; the server ignores it.
pub fn rig_local() -> openai::Client {
    openai::Client::from_url("EMPTY", "http://127.0.0.1:1234/v1/")
}
    