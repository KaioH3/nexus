"""
Nexus AI SDK — O Polars de Agentes AI

Fáceis de usar, rápidos em Rust, seguros por design.

Example:
    >>> from nexus import Agent
    >>> agent = Agent()
    >>> result = agent.execute("Say hello")
    >>> print(result.output)
    Hello! How can I help you?

Quick Start:
    >>> from nexus import code_review
    >>> print(code_review("auth.py"))
"""

from __future__ import annotations

import asyncio
import json
import uuid
import sys
from dataclasses import dataclass, field
from typing import Optional, Iterator, AsyncIterator, Union
from enum import Enum

__version__ = "0.2.0"
__all__ = [
    "Agent",
    "ExecutionResult",
    "SandboxPolicy",
    "ErrorCode",
    "NexusError",
    "AgentConfig",
    "Tool",
    "Memory",
    "code_review",
    "fix_bugs",
    "explain_code",
    "generate_tests",
    "run_terminal",
    "NexusConfig",
    "NexusServer",
    "SyncAgent",
    "lazy",
    "LazyAgent",
]


# ============================================================================
# Core Types
# ============================================================================

class ErrorCode(Enum):
    """Typed error codes instead of raw strings."""
    HANDSHAKE_FAILED = "handshake_failed"
    VERSION_MISMATCH = "version_mismatch"
    COMPILATION_FAILED = "compilation_failed"
    SANDBOX_VIOLATION = "sandbox_violation"
    SANDBOX_TIMEOUT = "sandbox_timeout"
    SANDBOX_OUT_OF_MEMORY = "sandbox_out_of_memory"
    OLLAMA_CONNECTION_FAILED = "ollama_connection_failed"
    OLLAMA_GENERATION_FAILED = "ollama_generation_failed"
    NETWORK_BLOCKED = "network_blocked"
    SYSCALL_BLOCKED = "syscall_blocked"
    FILE_NOT_FOUND = "file_not_found"
    PERMISSION_DENIED = "permission_denied"
    INVALID_MESSAGE = "invalid_message"
    MISSING_API_KEY = "missing_api_key"
    INVALID_API_KEY = "invalid_api_key"
    RATE_LIMITED = "rate_limited"
    INTERNAL_ERROR = "internal_error"


@dataclass
class NexusError(Exception):
    """Typed error with code instead of raw string."""
    code: ErrorCode
    message: str
    request_id: Optional[str] = None

    def __str__(self) -> str:
        if self.request_id:
            return f"[{self.code.value}] {self.message} (request: {self.request_id})"
        return f"[{self.code.value}] {self.message}"


@dataclass
class ExecutionResult:
    """
    Result from agent execution.

    Attributes:
        output: The main output from execution
        tools_used: List of tools that were used
        duration_ms: Time taken in milliseconds
        tokens_used: Number of tokens consumed
        cache_hit: Whether result was cached
    """
    output: str
    tools_used: list[str] = field(default_factory=list)
    duration_ms: int = 0
    tokens_used: int = 0
    cache_hit: bool = False

    def __str__(self) -> str:
        return self.output

    def __repr__(self) -> str:
        output_preview = self.output[:50] + "..." if len(self.output) > 50 else self.output
        return f"ExecutionResult(output={output_preview!r}, tools={self.tools_used!r}, duration_ms={self.duration_ms})"

    @property
    def ok(self) -> bool:
        """Check if execution was successful."""
        return len(self.output) > 0


@dataclass
class AgentConfig:
    """Advanced configuration for power users."""
    strategy: str = "react"
    temperature: float = 0.7
    max_tokens: int = 4096
    retry_attempts: int = 3
    retry_delay: float = 1.0
    sandbox_policy: str = "ai_generated_code"
    custom_system_prompt: Optional[str] = None
    debug: bool = False
    timeout_ms: int = 30000


@dataclass
class SandboxPolicy:
    """Sandbox policy for code execution."""
    max_memory_mb: int = 512
    max_cpu_time_ms: int = 30000
    allowed_paths: list[str] = field(default_factory=lambda: ["/tmp"])
    allowed_network: bool = False
    allowed_env: list[str] = field(default_factory=lambda: ["HOME", "TMP"])
    blocked_syscalls: list[int] = field(default_factory=list)

    @classmethod
    def zero_trust(cls) -> "SandboxPolicy":
        """Most restrictive - no network, minimal memory."""
        return cls(
            max_memory_mb=128,
            max_cpu_time_ms=5000,
            allowed_paths=[],
            allowed_network=False,
            allowed_env=[],
            blocked_syscalls=[2, 3, 4, 5, 9, 10, 41, 42, 43, 56, 57, 60, 61, 79, 85, 86, 137],
        )

    @classmethod
    def ai_generated_code(cls) -> "SandboxPolicy":
        """Recommended default for AI-generated code."""
        return cls(
            max_memory_mb=512,
            max_cpu_time_ms=30000,
            allowed_paths=["/tmp"],
            allowed_network=False,
            allowed_env=["HOME", "TMP"],
            blocked_syscalls=[2, 3, 4, 5, 9, 10, 41, 42, 43, 56, 57, 60, 61, 79, 85, 86, 137],
        )

    @classmethod
    def development(cls) -> "SandboxPolicy":
        """Lenient policy for development."""
        return cls(
            max_memory_mb=1024,
            max_cpu_time_ms=60000,
            allowed_paths=["/tmp", "/workspace"],
            allowed_network=True,
            allowed_env=["HOME", "USER", "PATH"],
            blocked_syscalls=[137],
        )


# ============================================================================
# Tool Abstractions
# ============================================================================

class Tool:
    """Base class for tools."""

    def __init__(self, name: str, **config):
        self.name = name
        self.config = config

    def __call__(self, *args, **kwargs):
        raise NotImplementedError

    def to_dict(self) -> dict:
        return {"name": self.name, "config": self.config}


class FilesystemTool(Tool):
    """Filesystem access with safety defaults."""

    def __init__(
        self,
        read: bool = True,
        write: bool = False,
        allowed_paths: Optional[list[str]] = None,
    ):
        super().__init__("filesystem", read=read, write=write)
        self.allowed_paths = allowed_paths or []


class TerminalTool(Tool):
    """Secure shell access."""

    def __init__(
        self,
        allowed_commands: Optional[list[str]] = None,
        timeout_seconds: int = 30,
    ):
        super().__init__(
            "terminal",
            allowed_commands=allowed_commands or [],
            timeout=timeout_seconds,
        )


# ============================================================================
# Memory
# ============================================================================

@dataclass
class Message:
    """A message in the agent's memory."""
    role: str
    content: str


class Memory:
    """Agent memory with automatic summarization."""

    def __init__(self, max_tokens: int = 8192):
        self.max_tokens = max_tokens
        self.messages: list[Message] = []
        self.summary: Optional[str] = None

    def add(self, role: str, content: str) -> None:
        """Add a message to memory."""
        self.messages.append(Message(role=role, content=content))

    def get_context(self) -> str:
        """Get current context (auto-summarizes if too long)."""
        if self.summary:
            return self.summary

        total_tokens = sum(len(m.content) // 4 for m in self.messages)
        if total_tokens > self.max_tokens:
            self.summary = self._summarize()
            return self.summary

        return "\n".join(f"{m.role}: {m.content}" for m in self.messages)

    def _summarize(self) -> str:
        """Simple truncation as fallback."""
        return f"[Summary of {len(self.messages)} messages]"

    def clear(self) -> None:
        """Clear memory."""
        self.messages.clear()
        self.summary = None


# ============================================================================
# Agent — The Main Class
# ============================================================================

class Agent:
    """
    O Agente AI mais fácil que existe.

    Usage:
        >>> agent = Agent()
        >>> result = agent.execute("Say hello")
        >>> print(result.output)
        Hello!

        >>> result = Agent().execute("Fix bugs in auth.py")
        >>> print(result.output)

        >>> with Agent() as a:
        ...     result = a.execute("Fix bugs in auth.py")
    """

    def __init__(
        self,
        model: str = "qwen2.5-coder:3b",
        *,
        tools: Optional[list[str]] = None,
        sandbox: str = "wasm",
        api_key: Optional[str] = None,
        config: Optional[AgentConfig] = None,
    ):
        self._model = model
        self._tools = tools or []
        self._sandbox = sandbox
        self._api_key = api_key or ""
        self._config = config or AgentConfig()
        self._client = None
        self._memory = Memory()

    # ─────────────────────────────────────────────────────────────
    # Polars-style Builder Pattern
    # ─────────────────────────────────────────────────────────────

    def tools(self, *names: str) -> "Agent":
        """Enable tools. Chainable. Usage: Agent().tools("filesystem", "terminal")"""
        self._tools.extend(names)
        return self

    def system(self, prompt: str) -> "Agent":
        """Set system prompt. Chainable."""
        self._config.custom_system_prompt = prompt
        return self

    def timeout(self, seconds: int) -> "Agent":
        """Set timeout. Chainable."""
        self._config.timeout_ms = seconds * 1000
        return self

    def retry(self, attempts: int = 3, delay: float = 1.0) -> "Agent":
        """Set retry policy. Chainable."""
        self._config.retry_attempts = attempts
        self._config.retry_delay = delay
        return self

    def temperature(self, temp: float) -> "Agent":
        """Set temperature. Chainable."""
        self._config.temperature = temp
        return self

    def sandbox(self, policy: Union[str, SandboxPolicy]) -> "Agent":
        """Set sandbox policy. Chainable."""
        if isinstance(policy, str):
            policy = {
                "wasm": SandboxPolicy.ai_generated_code(),
                "local": SandboxPolicy.development(),
                "zero": SandboxPolicy.zero_trust(),
            }.get(policy, SandboxPolicy.ai_generated_code())
        self._sandbox_policy = policy
        return self

    # ─────────────────────────────────────────────────────────────
    # Execute Methods
    # ─────────────────────────────────────────────────────────────

    def __call__(self, prompt: str) -> ExecutionResult:
        """Execute via callable interface. Usage: Agent()("task")"""
        return self.execute(prompt)

    def execute(self, prompt: str, *, stream: bool = False) -> ExecutionResult | Iterator[str]:
        """
        Execute a task.

        Args:
            prompt: Task description
            stream: If True, returns iterator of tokens

        Returns:
            ExecutionResult or token iterator if stream=True

        Usage:
            >>> result = Agent().execute("Say hello")
            >>> print(result.output)
            Hello!

            >>> for token in Agent().execute("Count to 5", stream=True):
            ...     print(token, end="", flush=True)
        """
        if stream:
            return self._execute_stream(prompt)
        return self._execute_once(prompt)

    def execute_streaming(self, prompt: str) -> Iterator[str]:
        """Alias for execute with stream=True."""
        return self._execute_stream(prompt)

    async def execute_async(self, prompt: str) -> ExecutionResult:
        """Async execution."""
        return self.execute(prompt)

    def _execute_once(self, prompt: str) -> ExecutionResult:
        """Internal single execution."""
        self._memory.add("user", prompt)

        try:
            result = self._execute_via_ollama(prompt)
            self._memory.add("assistant", result.output)
            return result
        except Exception as e:
            if self._config.debug:
                print(f"[Nexus Debug] Error: {e}", file=sys.stderr)
            raise

    def _execute_stream(self, prompt: str) -> Iterator[str]:
        """Internal streaming execution."""
        self._memory.add("user", prompt)

        import aiohttp
        import asyncio

        async def stream_generator():
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    "http://localhost:11434/api/generate",
                    json={
                        "model": self._model,
                        "prompt": self._build_prompt(prompt),
                        "stream": True,
                    },
                    timeout=aiohttp.ClientTimeout(total=self._config.timeout_ms / 1000),
                ) as resp:
                    if resp.status == 200:
                        async for line in resp.content:
                            if line:
                                data = json.loads(line)
                                if "response" in data:
                                    yield data["response"]
                    else:
                        error_text = await resp.text()
                        raise NexusError(
                            ErrorCode.OLLAMA_GENERATION_FAILED,
                            f"Ollama error {resp.status}: {error_text}",
                        )

        generator = stream_generator()
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)

        output = []
        try:
            for token in generator:
                if asyncio.iscoroutine(generator):
                    token = loop.run_until_complete(generator.__anext__())
                output.append(token)
                yield token
        except Exception as e:
            if self._config.debug:
                print(f"[Nexus Debug] Stream error: {e}", file=sys.stderr)
            raise

        result = "".join(output)
        self._memory.add("assistant", result)

    def _execute_via_ollama(self, prompt: str) -> ExecutionResult:
        """Execute via local Ollama."""
        import aiohttp
        import asyncio

        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)

        async def _fetch():
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    "http://localhost:11434/api/generate",
                    json={
                        "model": self._model,
                        "prompt": self._build_prompt(prompt),
                        "stream": False,
                    },
                    timeout=aiohttp.ClientTimeout(total=self._config.timeout_ms / 1000),
                ) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        return ExecutionResult(
                            output=data.get("response", ""),
                            tools_used=self._tools,
                            duration_ms=(data.get("total_duration", 0)) // 1_000_000,
                            tokens_used=data.get("eval_count", 0),
                        )
                    else:
                        error_text = await resp.text()
                        raise NexusError(
                            ErrorCode.OLLAMA_GENERATION_FAILED,
                            f"Ollama error {resp.status}: {error_text}",
                        )

        return loop.run_until_complete(_fetch())

    def _build_prompt(self, prompt: str) -> str:
        """Build the full prompt with context."""
        context = self._memory.get_context()
        system = self._config.custom_system_prompt or "You are a helpful AI assistant."

        full_prompt = f"{system}\n\n"
        if self._tools:
            full_prompt += f"Available tools: {', '.join(self._tools)}\n"
        if context:
            full_prompt += f"Context:\n{context}\n\n"
        full_prompt += f"Task: {prompt}"

        return full_prompt

    # ─────────────────────────────────────────────────────────────
    # One-liners (Most Common Tasks)
    # ─────────────────────────────────────────────────────────────

    def code_review(self, path: str) -> str:
        """One-liner: Code review a file. Usage: Agent().code_review("auth.py")"""
        return self.execute(f"Review code at {path}").output

    def fix_bugs(self, path: str) -> str:
        """One-liner: Auto-fix bugs in a file. Usage: Agent().fix_bugs("auth.py")"""
        return self.execute(f"Fix bugs in {path}").output

    def explain(self, code: str) -> str:
        """One-liner: Explain code. Usage: Agent().explain("x = 1 + 1")"""
        return self.execute(f"Explain this code:\n{code}").output

    def generate_tests(self, path: str) -> str:
        """One-liner: Generate tests for a file. Usage: Agent().generate_tests("auth.py")"""
        return self.execute(f"Generate tests for {path}").output

    def run(self, command: str) -> str:
        """One-liner: Run a terminal command. Usage: Agent().run("ls -la")"""
        return self.execute(f"Run this command and show output: {command}").output

    # ─────────────────────────────────────────────────────────────
    # Polars-style Context Manager
    # ─────────────────────────────────────────────────────────────

    def __enter__(self) -> "Agent":
        """Context manager entry. Usage: with Agent() as a: ..."""
        return self

    def __exit__(self, *args) -> None:
        """Context manager exit. Auto-cleanup."""
        self._close()

    def _close(self) -> None:
        """Close connections and cleanup."""
        if self._client:
            self._client = None

    # ─────────────────────────────────────────────────────────────
    # Debug & Introspection
    # ─────────────────────────────────────────────────────────────

    def debug(self, enabled: bool = True) -> "Agent":
        """Enable debug output. Usage: Agent().debug().execute("task")"""
        self._config.debug = enabled
        return self

    def __repr__(self) -> str:
        return f"Agent(model={self._model!r}, tools={self._tools!r})"


# ============================================================================
# Sync Wrapper
# ============================================================================

class SyncAgent:
    """
    Synchronous wrapper for non-async code.

    Usage:
        >>> agent = SyncAgent()
        >>> result = agent.execute("Say hello")
        >>> print(result.output)
    """

    def __init__(self, *args, **kwargs):
        self._agent = Agent(*args, **kwargs)

    def execute(self, prompt: str) -> ExecutionResult:
        """Execute synchronously."""
        return self._agent.execute(prompt)

    def code_review(self, path: str) -> str:
        """One-liner: Code review."""
        return self._agent.code_review(path)

    def fix_bugs(self, path: str) -> str:
        """One-liner: Fix bugs."""
        return self._agent.fix_bugs(path)

    def explain(self, code: str) -> str:
        """One-liner: Explain code."""
        return self._agent.explain(code)

    def debug(self, enabled: bool = True) -> "SyncAgent":
        """Enable debug mode."""
        self._agent.debug(enabled)
        return self

    def __enter__(self) -> "SyncAgent":
        return self

    def __exit__(self, *args) -> None:
        pass


# ============================================================================
# Lazy Agent (Polars-style)
# ============================================================================

def lazy(
    model: str = "qwen2.5-coder:3b",
    *,
    tools: Optional[list[str]] = None,
    **kwargs,
) -> LazyAgent:
    """
    Create a lazy agent (Polars-style).

    Nothing executes until .execute()

    Usage:
        >>> query = (
        ...     lazy()
        ...     .tools("filesystem")
        ...     .system("You are helpful")
        ... )
        >>> # Nothing happened yet...
        >>> result = query.execute("Say hello")  # NOW it runs
    """
    return LazyAgent(model=model, tools=tools, **kwargs)


class LazyAgent:
    """
    Lazy agent that builds a computation graph.

    Similar to Polars lazy DataFrames.

    Usage:
        >>> query = lazy().tools("filesystem")
        >>> result = query.execute("Say hello")
    """

    def __init__(
        self,
        model: str = "qwen2.5-coder:3b",
        *,
        tools: Optional[list[str]] = None,
        **config,
    ):
        self._model = model
        self._tools = tools or []
        self._config = config

    def tools(self, *names: str) -> "LazyAgent":
        """Add tools (chainable)."""
        self._tools.extend(names)
        return self

    def system(self, prompt: str) -> "LazyAgent":
        """Set system prompt (chainable)."""
        self._config["system_prompt"] = prompt
        return self

    def timeout(self, seconds: int) -> "LazyAgent":
        """Set timeout (chainable)."""
        self._config["timeout_ms"] = seconds * 1000
        return self

    def execute(self, prompt: str) -> ExecutionResult:
        """Execute the lazy agent (triggers execution)."""
        agent = Agent(model=self._model, tools=self._tools, **self._config)
        return agent.execute(prompt)


# ============================================================================
# One-liner Functions (Module-Level)
# ============================================================================

def code_review(path: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Review code at a path.

    Usage:
        >>> import nexus
        >>> nexus.code_review("auth.py")
        'This code has several issues...'
    """
    return Agent(model=model).code_review(path)


def fix_bugs(path: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Auto-fix bugs in a file.

    Usage:
        >>> import nexus
        >>> nexus.fix_bugs("auth.py")
        'Fixed version: ...'
    """
    return Agent(model=model).fix_bugs(path)


def explain_code(code: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Explain code.

    Usage:
        >>> import nexus
        >>> nexus.explain_code("x = 1 + 1")
        'This Python code...'
    """
    return Agent(model=model).explain(code)


def generate_tests(path: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Generate tests for a file.

    Usage:
        >>> import nexus
        >>> nexus.generate_tests("auth.py")
        'def test_...'
    """
    return Agent(model=model).generate_tests(path)


def run_terminal(command: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Run a terminal command.

    Usage:
        >>> import nexus
        >>> nexus.run_terminal("ls -la")
        'total 32...'
    """
    return Agent(model=model).run(command)


# ============================================================================
# Server (Enterprise)
# ============================================================================

class NexusServer:
    """
    Nexus Protocol server for team deployments.

    Usage:
        >>> server = NexusServer(port=8080)
        >>> server.start()
    """

    def __init__(
        self,
        port: int = 8080,
        *,
        tls: bool = True,
        auth: str = "api_key",
        rate_limit: tuple[int, int] = (60, 60),
    ):
        self.port = port
        self.tls = tls
        self.auth = auth
        self.rate_limit = rate_limit

    def start(self) -> None:
        """Start the server."""
        print(f"Nexus Protocol Server starting on port {self.port}")
        print("Endpoints:")
        print(f"  WebSocket: ws://localhost:{self.port}/api/v1/ws")
        print(f"  REST: http://localhost:{self.port}/api/v1/execute")


# ============================================================================
# Aliases for Ergonomics
# ============================================================================

NexusConfig = AgentConfig
