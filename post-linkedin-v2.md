MCP has 7,900 GitHub stars and a documented RCE they call "intended behavior." I built the fix.

Anthropic created MCP. Google adopted it. Thousands of production servers run it. Their specification allows arbitrary command execution with no sandbox and no authentication. Anthropic confirmed the behavior is by design.

I built Nexus Protocol. WASM sandbox. 17 dangerous syscalls blocked before your kernel sees them. Binary protocol — no JSON parsing overhead, no text negotiation between agent and system. Three sandbox policies configurable per execution: zero trust, AI-generated code, development. Prompt injection guard at the protocol layer, not a regex bolted on after the fact. Multi-language SDKs in Rust, Python, Go, and TypeScript. Local-first with Ollama — you bring your model, you control your costs, the protocol takes nothing. Apache 2.0 with explicit patent grant. 60 tests passing. Zero failures.

Sandboxing is not a feature. It is the foundation. If your agent can call execve() without a sandbox, you do not have a secure agent. You have a polite backdoor.

I am building Oscar CLI as the reference code agent implementation. If you want to test it before launch, DM me.

github.com/KaioH3/nexus

#AISecurity #MCP #NexusProtocol #Rust #OpenSource