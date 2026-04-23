//! Nexus Protocol TypeScript SDK
//!
//! TypeScript client for connecting to Nexus Protocol servers.

export type WasmRuntime = 'wasm3' | 'wasmer' | 'wasmtime' | 'native';

export interface Capabilities {
  wasmRuntimes: WasmRuntime[];
  ollama: boolean;
  ggufLoading: boolean;
  streaming: boolean;
  sandboxIsolation: boolean;
}

export interface SandboxPolicy {
  max_memory_mb: number;
  max_cpu_time_ms: number;
  allowed_paths: string[];
  allowed_network: boolean;
  allowed_env: string[];
  blocked_syscalls: number[];
}

export interface Message {
  type: string;
  request_id?: string;
  data?: string;
  code?: number;
  duration_ms?: number;
  exit_code?: number;
  execution_time_ms?: number;
  cache_hit?: boolean;
  message?: string;
  stderr?: string;
  token?: string;
  stats?: GenerationStats;
  models?: ModelInfo[];
}

export interface HandshakeMessage {
  type: 'handshake';
  version: string;
  api_key?: string;
  capabilities: Capabilities;
}

export interface HandshakeAckMessage {
  type: 'handshake_ack';
  session_id: string;
  server_version: string;
  capabilities: Capabilities;
}

export interface ExecuteMessage {
  type: 'execute';
  request_id: string;
  code: string;
  language: string;
  sandbox_policy: SandboxPolicy;
  model_hint?: string;
}

export interface ExecutionReadyMessage {
  type: 'execution_ready';
  request_id: string;
  wasm_module: Uint8Array;
}

export interface ExecutionResult {
  request_id: string;
  exit_code: number;
  stdout: string;
  stderr: string;
  execution_time_ms: number;
  cache_hit: boolean;
}

export interface GenerationStats {
  model: string;
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  duration_ms: number;
}

export interface ModelInfo {
  name: string;
  model: string;
  modified_at: string;
  size: number;
  digest: string;
}

export interface GenerateOptions {
  temperature?: number;
  top_p?: number;
  top_k?: number;
  num_predict?: number;
}

export class NexusClient {
  private ws: WebSocket | null = null;
  private sessionId: string | null = null;
  private url: string;
  private apiKey: string | null = null;
  private messageHandlers: Map<string, (msg: Message) => void> = new Map();

  constructor(url: string = 'ws://localhost:8080/api/v1/ws', apiKey?: string) {
    this.url = url;
    this.apiKey = apiKey || null;
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = async () => {
        try {
          // Send handshake
          const handshake: HandshakeMessage = {
            type: 'handshake',
            version: '0.1.0',
            api_key: this.apiKey || undefined,
            capabilities: {
              wasmRuntimes: ['wasm3', 'wasmer'],
              ollama: true,
              ggufLoading: true,
              streaming: true,
              sandboxIsolation: true,
            },
          };

          this.ws!.send(JSON.stringify(handshake));
        } catch (err) {
          reject(err);
        }
      };

      this.ws.onmessage = (event) => {
        const msg: Message = JSON.parse(event.data);

        if (msg.type === 'handshake_ack') {
          const ack = msg as unknown as HandshakeAckMessage;
          this.sessionId = ack.session_id;
          resolve();
        } else if (msg.type === 'error') {
          reject(new Error(msg.message));
        } else {
          // Call registered handler
          const handler = this.messageHandlers.get(msg.type);
          if (handler) {
            handler(msg);
          }
        }
      };

      this.ws.onerror = (event) => {
        reject(new Error('WebSocket error'));
      };
    });
  }

  on(type: string, handler: (msg: Message) => void): void {
    this.messageHandlers.set(type, handler);
  }

  async execute(code: string, language: string, policy?: SandboxPolicy): Promise<ExecutionResult> {
    if (!this.ws) {
      throw new Error('Not connected');
    }

    const requestId = crypto.randomUUID();
    const msg: ExecuteMessage = {
      type: 'execute',
      request_id: requestId,
      code,
      language,
      sandbox_policy: policy || SandboxPolicy.aiGeneratedCode(),
    };

    this.ws.send(JSON.stringify(msg));

    return new Promise((resolve, reject) => {
      let stdout = '';
      let stderr = '';
      let exitCode = 0;
      let durationMs = 0;

      const handleMessage = (msg: Message) => {
        switch (msg.type) {
          case 'stdout':
            stdout += msg.data || '';
            break;
          case 'stderr':
            stderr += msg.data || '';
            break;
          case 'exit':
            exitCode = msg.code || 0;
            durationMs = msg.duration_ms || 0;
            break;
          case 'execution_result':
            resolve({
              request_id: msg.request_id || requestId,
              exit_code: msg.exit_code || exitCode,
              stdout,
              stderr,
              execution_time_ms: msg.execution_time_ms || durationMs,
              cache_hit: msg.cache_hit || false,
            });
            this.messageHandlers.delete('execution_result');
            break;
          case 'error':
            reject(new Error(msg.message));
            this.messageHandlers.delete('execution_result');
            break;
        }
      };

      this.messageHandlers.set('execution_result', handleMessage);
    });
  }

  async *ollamaStream(model: string, prompt: string): AsyncGenerator<string> {
    if (!this.ws) {
      throw new Error('Not connected');
    }

    const requestId = crypto.randomUUID();
    const msg = {
      type: 'ollama_generate',
      request_id: requestId,
      model,
      prompt,
      options: {
        temperature: 0.8,
        top_p: 0.9,
        top_k: 40,
        num_predict: 256,
      },
      stream: true,
    };

    this.ws.send(JSON.stringify(msg));

    yield* this.createAsyncGenerator('ollama_token', 'ollama_done');
  }

  private async *createAsyncGenerator(
    tokenType: string,
    doneType: string
  ): AsyncGenerator<string> {
    if (!this.ws) {
      throw new Error('Not connected');
    }

    const queue: string[] = [];
    let done = false;

    const handler = (msg: Message) => {
      if (msg.type === tokenType) {
        queue.push(msg.token || '');
      } else if (msg.type === doneType) {
        done = true;
        this.messageHandlers.delete(doneType);
      }
    };

    this.messageHandlers.set(doneType, handler);

    while (!done) {
      if (queue.length > 0) {
        yield queue.shift()!;
      } else {
        await new Promise((resolve) => setTimeout(resolve, 10));
      }
    }

    // Drain remaining tokens
    while (queue.length > 0) {
      yield queue.shift()!;
    }
  }

  async close(): Promise<void> {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  get sessionId(): string | null {
    return this.sessionId;
  }
}

// SandboxPolicy helpers
export class SandboxPolicy {
  static zeroTrust(): SandboxPolicy {
    return {
      max_memory_mb: 128,
      max_cpu_time_ms: 5000,
      allowed_paths: [],
      allowed_network: false,
      allowed_env: [],
      blocked_syscalls: [
        2, 3, 4, 5, 9, 10, 41, 42, 43, 56, 57, 60, 61, 79, 85, 86, 137,
      ],
    };
  }

  static aiGeneratedCode(): SandboxPolicy {
    return {
      max_memory_mb: 512,
      max_cpu_time_ms: 30000,
      allowed_paths: ['/tmp'],
      allowed_network: false,
      allowed_env: ['HOME', 'TMP'],
      blocked_syscalls: [
        2, 3, 4, 5, 9, 10, 41, 42, 43, 56, 57, 60, 61, 79, 85, 86, 137,
      ],
    };
  }

  static development(): SandboxPolicy {
    return {
      max_memory_mb: 1024,
      max_cpu_time_ms: 60000,
      allowed_paths: ['/tmp', '/workspace'],
      allowed_network: true,
      allowed_env: ['HOME', 'USER', 'PATH'],
      blocked_syscalls: [137],
    };
  }
}

export default NexusClient;
