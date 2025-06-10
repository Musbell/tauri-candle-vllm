import { createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  // LLM specific state
  const [llmStatus, setLlmStatus] = createSignal("LLM not started.");
  const [prompt, setPrompt] = createSignal("");
  const [llmResponse, setLlmResponse] = createSignal("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  async function handleStartLlm() {
    setLlmStatus("Starting LLM, please wait...");
    try {
      await invoke("start_llm");
      setLlmStatus("LLM Ready. You can now ask questions.");
    } catch (error) {
      console.error("Failed to start LLM:", error);
      setLlmStatus(`Error starting LLM: ${error}`);
    }
  }

  async function handleAskQwen() {
    if (!prompt()) {
      setLlmResponse("Please enter a prompt.");
      return;
    }
    setLlmResponse("Thinking...");
    try {
      const response = await invoke("ask_qwen", { prompt: prompt() });
      setLlmResponse(response);
    } catch (error) {
      console.error("Failed to ask Qwen:", error);
      setLlmResponse(`Error asking Qwen: ${error}`);
    }
  }

  return (
    <main class="container">
      <h1>Welcome to Tauri + Solid + LLM</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={logo} class="logo solid" alt="Solid logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      {/* LLM Control Section */}
      <section class="llm-controls">
        <h2>LLM Interaction</h2>
        <div class="row">
          <button type="button" onClick={handleStartLlm}>
            Start LLM
          </button>
        </div>
        <p>{llmStatus()}</p>

        <form
          class="row"
          onSubmit={(e) => {
            e.preventDefault();
            handleAskQwen();
          }}
        >
          <input
            id="prompt-input"
            onChange={(e) => setPrompt(e.currentTarget.value)}
            placeholder="Enter your prompt for Qwen..."
            value={prompt()}
          />
          <button type="submit">Ask Qwen</button>
        </form>
        <p><strong>Response:</strong></p>
        <pre class="llm-response">{llmResponse()}</pre>
      </section>

      {/* Original Greet Section (optional, can be kept or removed) */}
      <section class="greet-section">
        <h2>Greeting Example</h2>
        <form
          class="row"
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
            value={name()}
          />
          <button type="submit">Greet</button>
        </form>
        <p>{greetMsg()}</p>
      </section>
    </main>
  );
}

export default App;
