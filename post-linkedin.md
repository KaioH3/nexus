# 🔵 LINKEDIN

---

MCP has 9+ documented CVEs, a 9.4 CVSS RCE in its official inspector, and Anthropic called it "intended behavior." I built the protocol that fixes it.

---

**The MCP security ledger (last 6 months):**

🔴 **CVE-2025-49596** — 9.4 CVSS. Remote code execution without authentication in MCP Inspector (Anthropic's official tool). 560 instances exposed on the internet. Attacker opens a webpage → arbitrary command runs on your machine.

🔴 **CVE-2025-6514** — 9.6 CVSS. RCE when any AI client connects to a malicious MCP server. Arbitrary command via the `open()` function.

🔴 **CVE-2026-30615** — Zero-click prompt injection. Edits MCP JSON config without user interaction.

🔴 **CVE-2025-68143/45/44** — Arbitrary code execution, file read, and file deletion via malicious README. Affects Git MCP Server (Anthropic's official project).

🔴 **CVE-2025-34072** — Data exfiltration via Slack MCP Server (Anthropic's official integration).

That's just the official projects. Add 10+ more CVEs in projects that inherited MCP code: Langchain-Chatchat, LiteLLM, GPT Researcher, Agent Zero, Fay Framework, DocsGPT, Flowise, and more.

**200,000+ vulnerable instances. 150 million downloads. And counting.**

---

**The design flaw that creates all of it:**

MCP's STDIO interface executes arbitrary commands without sanitization. You pass a prompt. MCP tries to run it. It fails. Returns an error. But the command already executed.

Anthropic confirmed this. Their response to the security report:

> *"Anthropic confirmed the behavior is by design and declined to modify the protocol, stating the STDIO execution model represents a secure default and that sanitization is the developer's responsibility."*

Translation: "We shipped RCE by design. It's your problem now."

---

I spent the last weeks building the answer.

**Nexus Protocol.** A sandbox-first standard for agent↔system communication. Designed from scratch to replace MCP — not wrap it in security duct tape.

Oscar CLI is the reference implementation (1.2MB, pure Rust, Apache 2.0). But the product is the protocol.

---

**The test I ran live — and almost didn't post because it's embarrassingly obvious:**

**MCP:**
```
Agent: "delete old_system.sql"
→ ✅ Success. File's gone. No prompt. No confirmation. No sandbox.
```

**Nexus Protocol (via Oscar CLI):**
```
oscar run --sandbox max "delete old_system.sql"
→ ⛔ Syscall 87 (unlink) blocked by WASM Sandbox.
```

Same command. Two protocols. One deletes. The other blocks at the kernel boundary.

---

**What Nexus Protocol delivers today — not a whitepaper, not a waitlist:**

⚡ **5ms latency.** MCP: 400ms. 80x gap because Nexus uses binary protocol, not JSON negotiation.

🔒 **17 dangerous syscalls blocked.** Unlink, execve, mount, chmod, and 13 more. Configurable policies (none, read-only, standard, max).

🛡️ **WASM sandbox as core protocol feature.** Not a wrapper. Not a best practice. Not "developer's responsibility." Enforced at the protocol layer — same way TLS is core to HTTPS, not a "recommended addition."

🆓 **Local with Ollama.** Zero API cost. Zero data exfiltration risk. 14 models supported, smallest is 397MB.

📜 **Prompt injection guard.** Structural validation at the message layer. MCP's prompt injection becomes code execution. Nexus Protocol's prompt injection gets blocked.

📜 **Apache 2.0.** Explicit patent grant. MCP is MIT. When the agent infrastructure patent wars heat up — and with 9+ CVEs, that friction is coming — Apache 2.0 projects don't flinch.

Rate limiting, connection pooling, binary protocol, WASM sandbox, multi-language SDKs. All in the repo. 37 tests passing. Compiles in 43 seconds.

---

**The thesis:**

We've spent 18 months obsessed with prompt engineering, model alignment, and RLHF. All of it runs in userspace. None of it touches syscalls.

MCP proved it: you can have perfect model alignment and still ship RCE by design. Because the vulnerability isn't in the model. It's in the protocol between the agent and the system.

Agent security isn't a prompt problem. It's an operating system problem.

---

**The license isn't a legal detail:**

MIT says "use this, no warranty, good luck." Apache 2.0 says "here's an explicit patent grant, attribution protection, and a legal framework tested by Google, Amazon, and Microsoft."

When MCP inevitably hits patent friction — and with 9+ CVEs and 200k vulnerable instances, it will — Apache 2.0 projects keep standing.

---

**What I'm actually launching:**

Not a CLI. A protocol designed to make sandboxing the default, not the afterthought. Oscar CLI is the reference implementation. The spec is open. The license is permissive. The bet is that insecure agents become unshippable the same way unencrypted HTTP did.

MCP shipped RCE and called it "intended behavior." Nexus Protocol ships security and calls it the protocol.

---

**Test it in 30 seconds. No signup. No cloud. No excuses:**

```bash
git clone https://github.com/KaioH3/nexus
cargo build --release
oscar run --sandbox max "rm -rf /"
```

Spoiler: your files will still be there. The logs show exactly which syscalls were blocked and why.

---

If you're building or deploying AI agents, ask yourself one question:

**Do you have a sandbox between your agent and your filesystem, or are you trusting a protocol that called RCE "intended behavior"?**

MCP's answer is documented. Mine compiles in 43 seconds.

---

## Comments thread (post 1-2 hours later)

**Comment 1:**
The pattern I'm seeing:

MCP advocates: "Just use best practices, add a security layer, trust your agents."
Also MCP: *has 9+ CVEs, including a 9.4 CVSS RCE in the official inspector, confirmed "by design"*

Best practices don't replace architecture. A protocol that calls RCE "intended behavior" doesn't get safer with wrappers.

---

**Comment 2:**
To be clear about the "just wrap MCP in security" argument:

You can add a sandbox around MCP. You can add rate limiting. You can add prompt filters.

But that's like saying you can make HTTP secure by adding TLS as a middleware layer instead of building it into the protocol.

The protocol shapes the ecosystem. MCP's "intended behavior" is baked into how every SDK, every tool, every template implements it. Security as an afterthought stays afterthought.

Nexus Protocol was designed sandbox-first. That's not a feature. That's the foundation.

---

**Comment 3:**
The real cost of "intended behavior":

200,000+ vulnerable instances.
150 million downloads.
10+ projects (Langchain-Chatchat, LiteLLM, GPT Researcher, Agent Zero, Fay Framework, DocsGPT, Flowise...) inheriting the same code and the same vulnerabilities.

This isn't a patching problem. This is a supply chain problem.

When you build on MCP, you're not building on a protocol. You're building on a liability with 9+ CVEs and a vendor who refuses to fix the architecture.

Nexus Protocol is the supply chain fix.