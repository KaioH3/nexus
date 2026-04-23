MCP became the standard with 84,000 GitHub stars and a documented hole: zero sandbox, zero authentication, RCE by design.

I built Nexus Protocol to fix that at the protocol layer, not with wrappers.

What ships today:
18 syscalls blocked. WASM sandbox. Binary protocol. Three sandbox policies. Prompt injection guard. SDKs in Rust, Python, Go, TypeScript. Apache 2.0. 60 tests passing.

The demo that matters:
MCP receives "rm -rf /" and executes it. Intended behavior.
Nexus Protocol blocks syscall 87 (unlink) before the kernel sees it.

Repo: github.com/KaioH3/nexus
Apache 2.0. Read the spec. Break the sandbox. Tell me what you find.