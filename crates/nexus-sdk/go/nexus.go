//! Nexus Protocol Go SDK
//!
//! Go client for connecting to Nexus Protocol servers.

package nexus

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"nhooyr.io/websocket"
	"time"

	"github.com/google/uuid"
)

type Client struct {
	baseURL    string
	apiKey     string
	httpClient *http.Client
	wsConn     *websocket.Conn
	sessionID  uuid.UUID
}

type Config struct {
	BaseURL string
	APIKey  string
}

func NewClient(cfg Config) (*Client, error) {
	if cfg.BaseURL == "" {
		cfg.BaseURL = "http://localhost:8080"
	}

	return &Client{
		baseURL: cfg.BaseURL,
		apiKey: cfg.APIKey,
		httpClient: &http.Client{
			Timeout: 60 * time.Second,
		},
	}, nil
}

func (c *Client) Connect(ctx context.Context) error {
	url := c.baseURL + "/api/v1/ws"

	conn, _, err := websocket.Dial(ctx, url, nil)
	if err != nil {
		return fmt.Errorf("WebSocket connection failed: %w", err)
	}

	c.wsConn = conn

	// Send handshake
	handshake := HandshakeMessage{
		Type:          "handshake",
		Version:       "0.1.0",
		APIKey:        c.apiKey,
		Capabilities: Capabilities{
			WasmRuntimes:     []string{"wasm3", "wasmer"},
			Ollama:           true,
			GgufLoading:      true,
			Streaming:        true,
			SandboxIsolation: true,
		},
	}

	if err := c.wsConn.Write(ctx, websocket.MessageJSON, handshake); err != nil {
		return fmt.Errorf("handshake failed: %w", err)
	}

	// Read handshake ack
	_, msg, err := c.wsConn.Read(ctx)
	if err != nil {
		return fmt.Errorf("handshake ack failed: %w", err)
	}

	var ack HandshakeAckMessage
	if err := json.Unmarshal(msg, &ack); err != nil {
		return fmt.Errorf("invalid handshake ack: %w", err)
	}

	c.sessionID = ack.SessionID
	return nil
}

func (c *Client) Execute(ctx context.Context, code string, language string, policy SandboxPolicy) (*ExecutionResult, error) {
	requestID := uuid.New().String()

	msg := ExecuteMessage{
		Type:          "execute",
		RequestID:     requestID,
		Code:          code,
		Language:      language,
		SandboxPolicy: policy,
	}

	if err := c.wsConn.Write(ctx, websocket.MessageJSON, msg); err != nil {
		return nil, fmt.Errorf("execute write failed: %w", err)
	}

	var result ExecutionResult
	stdout := ""

	for {
		_, msg, err := c.wsConn.Read(ctx)
		if err != nil {
			return nil, fmt.Errorf("read failed: %w", err)
		}

		var resp Message
		if err := json.Unmarshal(msg, &resp); err != nil {
			continue
		}

		switch resp.Type {
		case "stdout":
			stdout += string(resp.Data)
		case "exit":
			result.ExitCode = resp.Code
			result.DurationMs = resp.DurationMs
		case "execution_result":
			result.RequestID = resp.RequestID
			result.ExitCode = resp.ExitCode
			result.Stdout = stdout
			result.Stderr = string(resp.Stderr)
			result.ExecutionTimeMs = resp.ExecutionTimeMs
			result.CacheHit = resp.CacheHit
			return &result, nil
		case "error":
			return nil, fmt.Errorf("execution error: %s", resp.Message)
		}
	}
}

func (c *Client) OllamaGenerate(ctx context.Context, model, prompt string) (<-chan string, error) {
	requestID := uuid.New().String()

	msg := OllamaGenerateMessage{
		Type:      "ollama_generate",
		RequestID: requestID,
		Model:     model,
		Prompt:    prompt,
		Options: GenerateOptions{
			Temperature: 0.8,
			TopP:        0.9,
			TopK:        40,
			NumPredict:  256,
		},
		Stream: true,
	}

	if err := c.wsConn.Write(ctx, websocket.MessageJSON, msg); err != nil {
		return nil, err
	}

	tokens := make(chan string, 100)

	go func() {
		defer close(tokens)
		for {
			_, msg, err := c.wsConn.Read(ctx)
			if err != nil {
				return
			}

			var resp OllamaMessage
			if err := json.Unmarshal(msg, &resp); err != nil {
				continue
			}

			switch resp.Type {
			case "ollama_token":
				tokens <- resp.Token
			case "ollama_done":
				return
			}
		}
	}()

	return tokens, nil
}

func (c *Client) Close() error {
	if c.wsConn != nil {
		return c.wsConn.Close(websocket.StatusNormalClosure, "")
	}
	return nil
}

// Message types

type Message struct {
	Type      string `json:"type"`
	RequestID string `json:"request_id,omitempty"`
	Data      string `json:"data,omitempty"`
	Stderr    []byte `json:"stderr,omitempty"`
	Code      int    `json:"code,omitempty"`
	DurationMs uint64 `json:"duration_ms,omitempty"`
	ExitCode    int    `json:"exit_code,omitempty"`
	ExecutionTimeMs uint64 `json:"execution_time_ms,omitempty"`
	CacheHit   bool   `json:"cache_hit,omitempty"`
	Message   string `json:"message,omitempty"`
}

type HandshakeMessage struct {
	Type          string       `json:"type"`
	Version       string       `json:"version"`
	APIKey        string       `json:"api_key,omitempty"`
	Capabilities  Capabilities `json:"capabilities"`
}

type Capabilities struct {
	WasmRuntimes     []string `json:"wasm_runtimes"`
	Ollama           bool     `json:"ollama"`
	GgufLoading      bool     `json:"gguf_loading"`
	Streaming        bool     `json:"streaming"`
	SandboxIsolation bool     `json:"sandbox_isolation"`
}

type HandshakeAckMessage struct {
	Type          string       `json:"type"`
	SessionID     uuid.UUID   `json:"session_id"`
	ServerVersion string       `json:"server_version"`
	Capabilities  Capabilities `json:"capabilities"`
}

type ExecuteMessage struct {
	Type          string        `json:"type"`
	RequestID     string        `json:"request_id"`
	Code          string        `json:"code"`
	Language      string        `json:"language"`
	SandboxPolicy SandboxPolicy `json:"sandbox_policy"`
	ModelHint     *string       `json:"model_hint,omitempty"`
}

type SandboxPolicy struct {
	MaxMemoryMB     uint64   `json:"max_memory_mb"`
	MaxCPUTimeMs    uint64   `json:"max_cpu_time_ms"`
	AllowedPaths    []string `json:"allowed_paths"`
	AllowedNetwork  bool     `json:"allowed_network"`
	AllowedEnv      []string `json:"allowed_env"`
	BlockedSyscalls []uint32 `json:"blocked_syscalls"`
}

func (p *SandboxPolicy) ZeroTrust() *SandboxPolicy {
	p.MaxMemoryMB = 128
	p.MaxCPUTimeMs = 5000
	p.AllowedPaths = []string{}
	p.AllowedNetwork = false
	p.AllowedEnv = []string{}
	return p
}

func (p *SandboxPolicy) AIGeneratedCode() *SandboxPolicy {
	p.MaxMemoryMB = 512
	p.MaxCPUTimeMs = 30000
	p.AllowedPaths = []string{"/tmp"}
	p.AllowedNetwork = false
	p.AllowedEnv = []string{"HOME", "TMP"}
	return p
}

func (p *SandboxPolicy) Development() *SandboxPolicy {
	p.MaxMemoryMB = 1024
	p.MaxCPUTimeMs = 60000
	p.AllowedPaths = []string{"/tmp", "/workspace"}
	p.AllowedNetwork = true
	p.AllowedEnv = []string{"HOME", "USER", "PATH"}
	return p
}

type OllamaGenerateMessage struct {
	Type      string           `json:"type"`
	RequestID string           `json:"request_id"`
	Model     string           `json:"model"`
	Prompt    string           `json:"prompt"`
	Options   GenerateOptions  `json:"options"`
	Stream    bool             `json:"stream"`
}

type GenerateOptions struct {
	Temperature *float64 `json:"temperature,omitempty"`
	TopP        *float64 `json:"top_p,omitempty"`
	TopK        *uint32  `json:"top_k,omitempty"`
	NumPredict  *uint32  `json:"num_predict,omitempty"`
}

type OllamaMessage struct {
	Type   string `json:"type"`
	Token  string `json:"token,omitempty"`
	Model  string `json:"model,omitempty"`
	Stats  GenerationStats `json:"stats,omitempty"`
}

type GenerationStats struct {
	Model           string `json:"model"`
	PromptTokens    uint32 `json:"prompt_tokens"`
	CompletionTokens uint32 `json:"completion_tokens"`
	TotalTokens     uint32 `json:"total_tokens"`
	DurationMs      uint64 `json:"duration_ms"`
}

type ExecutionResult struct {
	RequestID       string `json:"request_id"`
	ExitCode       int    `json:"exit_code"`
	Stdout         string `json:"stdout"`
	Stderr         string `json:"stderr"`
	ExecutionTimeMs uint64 `json:"execution_time_ms"`
	CacheHit       bool   `json:"cache_hit"`
}
