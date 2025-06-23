import { createSignal, createEffect, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  // Chat conversation history
  const [messages, setMessages] = createSignal([
    {
      role: "assistant",
      content: "Hello! I'm Message Mate, your professional and efficient AI writing assistant. How can I help you craft your message today? Please provide some details about your request.",
      timestamp: new Date()
    }
  ]);

  const [inputMessage, setInputMessage] = createSignal("");
  const [isLoading, setIsLoading] = createSignal(false);
  const [llmStatus, setLlmStatus] = createSignal("offline");

  // Reference to message container for auto-scrolling
  let messageContainer;

  // Scroll to bottom when new messages arrive
  createEffect(() => {
    if (messages().length && messageContainer) {
      setTimeout(() => {
        messageContainer.scrollTop = messageContainer.scrollHeight;
      }, 100);
    }
  });

  onMount(async () => {
    // Start LLM automatically when app loads
    handleStartLlm();
  });

  async function handleStartLlm() {
    setLlmStatus("starting");
    try {
      await invoke("start_llm");
      setLlmStatus("online");
    } catch (error) {
      console.error("Failed to start LLM:", error);
      setLlmStatus("error");

      setMessages(prev => [...prev, {
        role: "system",
        content: "Error starting the AI assistant. Please try again later.",
        timestamp: new Date(),
        error: true
      }]);
    }
  }

  async function handleSendMessage(e) {
    e.preventDefault();

    if (!inputMessage().trim()) return;

    const userMessage = inputMessage();

    // Add user message to chat
    setMessages(prev => [...prev, {
      role: "user",
      content: userMessage,
      timestamp: new Date()
    }]);

    // Clear input
    setInputMessage("");

    // Show loading indicator
    setIsLoading(true);

    try {
      // Call backend
      const response = await invoke("ask_qwen", { prompt: userMessage });

      // Add AI response to chat
      setMessages(prev => [...prev, {
        role: "assistant",
        content: response,
        timestamp: new Date()
      }]);
    } catch (error) {
      console.error("Failed to get response:", error);

      // Add error message
      setMessages(prev => [...prev, {
        role: "system",
        content: `Error: ${error}`,
        timestamp: new Date(),
        error: true
      }]);
    } finally {
      setIsLoading(false);
    }
  }

  // Format timestamp to readable time
  function formatTime(date) {
    return new Date(date).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  return (
    <div class="chat-container">
      {/* Header */}
      <header class="chat-header">
        <div class="status-indicator">
          <div class={`status-dot ${llmStatus()}`}></div>
          <span>
            {llmStatus() === "offline" && "Offline"}
            {llmStatus() === "starting" && "Starting..."}
            {llmStatus() === "online" && "Online"}
            {llmStatus() === "error" && "Error"}
          </span>
        </div>
        <h1>Message Mate - Professional AI Writing Assistant</h1>
        <button class="restart-button" onClick={handleStartLlm} title="Restart AI">
          ⟳
        </button>
      </header>

      {/* Message Container */}
      <div class="messages-container" ref={messageContainer}>
        {messages().map((message) => (
          <div class={`message ${message.role} ${message.error ? 'error' : ''}`}>
            {message.role === "assistant" && (
              <div class="avatar">🤖</div>
            )}
            <div class="message-content">
              <div class="message-text">{message.content}</div>
              <div class="message-time">{formatTime(message.timestamp)}</div>
            </div>
            {message.role === "user" && (
              <div class="avatar user">👤</div>
            )}
          </div>
        ))}

        {/* Loading indicator */}
        {isLoading() && (
          <div class="message assistant">
            <div class="avatar">🤖</div>
            <div class="message-content">
              <div class="typing-indicator">
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Input Area */}
      <form class="input-area" onSubmit={handleSendMessage}>
        <input
          type="text"
          value={inputMessage()}
          onInput={(e) => setInputMessage(e.target.value)}
          placeholder="Type a message..."
          disabled={isLoading() || llmStatus() !== "online"}
        />
        <button 
          type="submit" 
          disabled={isLoading() || !inputMessage().trim() || llmStatus() !== "online"}
        >
          {isLoading() ? "..." : "Send"}
        </button>
      </form>
    </div>
  );
}

export default App;
