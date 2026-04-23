MCP became the standard with thousands of production servers and a documented RCE they call "intended behavior." I built the fix. It's called Nexus Protocol.

Anthropic created MCP. Google adopted it. MCP's specification allows arbitrary command execution with no sandbox and no authentication. Anthropic confirmed the behavior is by design.

But the RCE is only half the problem. The other half is vendor lock-in. MCP was built by an API company. The default path is proprietary models, per-token billing, and praying the pricing page doesn't change. You do not own the stack. You rent it.

Nexus Protocol is LLM-agnostic by design. Ollama local. Groq. OpenAI. Your choice. Your model runs on your hardware or your cloud account with no per-request protocol tax. Switch providers without changing a line of agent code.

There is one interface. It works the same whether your model runs on a 397MB local quant or a 70B cloud instance. The protocol does not care. That is the architecture MCP should have shipped.

What is in the repo today:

WASM sandbox. 17 dangerous syscalls blocked before your kernel sees them. Binary protocol with measured 1.6x faster serialization than JSON. API key validation with constant-time hash comparison. Prompt injection guard at the protocol layer. Rate limiting. Connection pooling. Multi-language SDKs in Rust, Python, Go, and TypeScript. Three sandbox policies configurable per execution. Apache 2.0 with explicit patent grant. 60 tests passing. Zero failures.

Sandboxing is not a feature. It is the foundation. If your agent can call execve() without a sandbox, you do not have a secure agent. You have a polite backdoor.

The protocol is on GitHub. I am building Oscar CLI as the reference code agent implementation. DM me if you want to test it before launch.

github.com/KaioH3/nexus

#AISecurity #MCP #NexusProtocol #Rust #OpenSource