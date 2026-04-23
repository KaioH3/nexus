# Nexus Python SDK — Polars/FastAPI-Level Ergonomics

**The North Star:** Python developers should be able to use Nexus in **under 30 seconds** with **zero configuration** and **maximum delight**.

---

## Design Principles

### 1. Zero Configuration (Works Out of the Box)

```python
# Install: pip install nexus-ai
# That's it. Nothing else.

from nexus import Agent

agent = Agent()  # No config needed, defaults Just Work
result = agent("Say hello")  # Agent is callable!
print(result)
```

### 2. Progressive Disclosure (Simple to Powerful)

```python
# Level 1: One-liner (most users)
result = Agent().execute("Fix bugs in auth.py")

# Level 2: Configure (power users)
result = (
    Agent(model="qwen2.5-coder:3b")
    .tools("filesystem", "terminal")
    .sandbox("wasm")
    .execute("Fix bugs in auth.py")
)

# Level 3: Full control (experts)
config = AgentConfig(
    model="qwen2.5-coder:3b",
    strategy="cot",  # Chain of thought
    temperature=0.3,
    max_tokens=4096,
    retry_attempts=3,
)
result = Agent(config=config).execute("Complex task")
```

### 3. Polars-Style Lazy Evaluation

```python
# DOES NOT EXECUTE until you call .collect() or .execute()
# (This is the Polars magic)

query = (
    Agent()
    .tools("filesystem", "terminal")
    .system("You are a senior developer")
    .timeout(30)
)

# Nothing happened yet...
result = query.execute("Fix bugs in auth.py")  # NOW it executes

# Streaming version
for token in query.stream("Write tests for auth.py"):
    print(token, end="", flush=True)
```

### 4. Pythonic Patterns

```python
# Context manager (like Polars)
with Agent() as a:
    a.tools("filesystem")
    result = a.execute("Fix bugs")

# Pipe operator (Unix-style)
result = (
    Agent()
    | "Fix bugs in auth.py"
    | {"language": "python"}
    | Execute()
)

# Generator expressions (lazy)
results = [a.execute(f"Review {f}") for f in files]  # Sequential
results = [a.execute(f"Review {f}") async for f in files]  # Async

# Async first class
import asyncio
async def main():
    result = await Agent().execute_async("Say hello")
asyncio.run(main())
```

---

## API Design — The Complete Picture

### The `nexus` Module (One Import Everything)

```python
"""
Nexus AI SDK — O Polars de Agentes AI

Fáceis de usar, rápidos em Rust, seguros por design.

Example:
    >>> from nexus import Agent
    >>> agent = Agent()
    >>> print(agent("Hello world"))

Quick Start:
    >>> from nexus import Agent
    >>> result = Agent().execute("Say hello")
    >>> print(result.output)
"""

from __future__ import annotations

# Core classes — the ONLY things most users need
from nexus._native import Agent, ExecutionResult
from nexus._native import SandboxPolicy, ErrorCode, NexusError

# Convenience — one-liners
from nexus._native import (
    code_review,
    fix_bugs,
    explain_code,
    generate_tests,
    run_terminal,
)

# Lazy API
from nexus._lazy import LazyAgent, lazy

# Re-exports
__all__ = [
    # Core
    "Agent",
    "ExecutionResult",
    "SandboxPolicy",
    "ErrorCode",
    "NexusError",

    # One-liners
    "code_review",
    "fix_bugs",
    "explain_code",
    "generate_tests",
    "run_terminal",

    # Lazy
    "LazyAgent",
    "lazy",

    # Utils
    "NexusConfig",
    "Tool",
    "Memory",
]
```

### The Agent Class (Main Entry Point)

```python
class Agent:
    """
    O Agente AI mais fácil que existe.

    Usage:
        >>> agent = Agent()
        >>> result = agent("Say hello")

        >>> result = Agent().execute("Say hello")

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
        """
        Create an agent.

        Args:
            model: Model to use (default: qwen2.5-coder:3b)
                - "qwen2.5-coder:3b" (fastest, best quality/speed)
                - "deepseek-coder:1.3b" (balanced)
                - "llama3.2:1b" (local, lightweight)
                - "gpt-4" (cloud, most capable)

            tools: List of tools to enable
                - "filesystem" (read files)
                - "terminal" (run commands)
                - "websearch" (search the web)
                - "memory" (persistent context)

            sandbox: Execution sandbox
                - "wasm" (default, safe)
                - "local" (faster, less safe)
                - "cloud" (remote execution)

            api_key: API key for cloud services (optional)
            config: Advanced configuration
        """
        self._model = model
        self._tools = tools or []
        self._sandbox = sandbox
        self._api_key = api_key or ""
        self._config = config or AgentConfig()
        self._client = None

    # ─────────────────────────────────────────────────────────────
    # Polars-style Builder Pattern
    # ─────────────────────────────────────────────────────────────

    def tools(self, *tools: str) -> "Agent":
        """Enable tools. Chainable."""
        self._tools.extend(tools)
        return self

    def system(self, prompt: str) -> "Agent":
        """Set system prompt. Chainable."""
        self._config.system_prompt = prompt
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

    # ─────────────────────────────────────────────────────────────
    # Execute Methods
    # ─────────────────────────────────────────────────────────────

    def __call__(self, prompt: str) -> ExecutionResult:
        """Execute via callable interface."""
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
            1... 2... 3... 4... 5...
        """
        if stream:
            return self._execute_stream(prompt)
        return self._execute_once(prompt)

    async def execute_async(self, prompt: str) -> ExecutionResult:
        """Async execution."""
        return await asyncio.to_thread(self.execute, prompt)

    # ─────────────────────────────────────────────────────────────
    # One-liners (Most Common Tasks)
    # ─────────────────────────────────────────────────────────────

    def code_review(self, path: str) -> str:
        """One-liner: Code review a file."""
        return self.execute(f"Review code at {path}").output

    def fix_bugs(self, path: str) -> str:
        """One-liner: Auto-fix bugs in a file."""
        return self.execute(f"Fix bugs in {path}").output

    def explain(self, code: str) -> str:
        """One-liner: Explain code."""
        return self.execute(f"Explain this code:\n{code}").output

    def generate_tests(self, path: str) -> str:
        """One-liner: Generate tests for a file."""
        return self.execute(f"Generate tests for {path}").output

    def run(self, command: str) -> str:
        """One-liner: Run a terminal command."""
        return self.execute(f"Run: {command}").output

    # ─────────────────────────────────────────────────────────────
    # Polars-style Context Manager
    # ─────────────────────────────────────────────────────────────

    def __enter__(self) -> "Agent":
        """Context manager entry."""
        self._connect()
        return self

    def __exit__(self, *args) -> None:
        """Context manager exit. Auto-closes."""
        self._close()

    # ─────────────────────────────────────────────────────────────
    # Debug & Introspection
    # ─────────────────────────────────────────────────────────────

    def debug(self, enabled: bool = True) -> "Agent":
        """Enable debug output."""
        self._config.debug = enabled
        return self

    def __repr__(self) -> str:
        return f"Agent(model={self._model!r}, tools={self._tools!r})"
```

### The ExecutionResult (Rich Return Type)

```python
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
        return (
            f"ExecutionResult("
            f"output={self.output[:50]!r}..."
            f"tools={self.tools_used!r}, "
            f"duration_ms={self.duration_ms})"
        )

    @property
    def ok(self) -> bool:
        """Check if execution was successful."""
        return len(self.output) > 0

    @property
    def error(self) -> Optional[str]:
        """Get error message if any."""
        return None  # For now, successful executions don't have errors
```

### The Lazy Agent (Polars-Style)

```python
def lazy(
    model: str = "qwen2.5-coder:3b",
    *,
    tools: Optional[list[str]] = None,
    **kwargs,
) -> LazyAgent:
    """
    Create a lazy agent (Polars-style).

    Nothing executes until .execute() or .collect()

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

    Similar to Polars lazy DataFrames:
        >>> df = pl.read_csv("file.csv").lazy()
        >>> result = df.filter(pl.col("age") > 21).collect()

    Usage:
        >>> agent = lazy().tools("filesystem")
        >>> result = agent.execute("Say hello")  # Executes
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

    def collect(self) -> "LazyAgent":
        """Alias for execute (Polars-style)."""
        raise NotImplementedError("Use .execute(prompt)")
```

### One-liner Functions (Module-Level Convenience)

```python
def code_review(path: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Review code at a path.

    Usage:
        >>> import nexus
        >>> nexus.code_review("auth.py")
        # Returns: "This code has X issues: ..."
    """
    return Agent(model=model).code_review(path)


def fix_bugs(path: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Auto-fix bugs in a file.

    Usage:
        >>> import nexus
        >>> nexus.fix_bugs("auth.py")
        # Returns: fixed code
    """
    return Agent(model=model).fix_bugs(path)


def explain_code(code: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Explain code.

    Usage:
        >>> import nexus
        >>> nexus.explain_code("x = 1 + 1")
        # Returns: "This Python code..."
    """
    return Agent(model=model).explain(code)


def generate_tests(path: str, model: str = "qwen2.5-coder:3b") -> str:
    """
    One-liner: Generate tests for a file.

    Usage:
        >>> import nexus
        >>> nexus.generate_tests("auth.py")
        # Returns: test code
    """
    return Agent(model=model).generate_tests(path)
```

---

## Usage Examples (The "Wow" Moments)

### 1. Simplest Possible Use

```python
# 3 linhas. 30 segundos. Feito.
pip install nexus-ai

python << 'EOF'
from nexus import Agent
print(Agent()("Say hello"))
EOF
# Output: Hello! How can I help you?
```

### 2. File Operations

```python
from nexus import Agent

with Agent() as a:
    a.tools("filesystem")

    # Read and review a file
    result = a.execute("Review auth.py and fix any bugs")
    print(result.output)

    # Write fixed version
    a.execute(f"Rewrite auth.py with fixes:\n{result.output}")
```

### 3. Terminal Commands

```python
from nexus import Agent

result = Agent().run("git status")
print(result)

result = Agent().run("cargo build --release")
print(result)
```

### 4. Code Review

```python
from nexus import code_review

review = code_review("auth.py")
print(review)
```

### 5. Streaming Output

```python
from nexus import Agent

for token in Agent().execute("Write a story about robots", stream=True):
    print(token, end="", flush=True)
```

### 6. Async Pipeline

```python
import asyncio
from nexus import Agent

async def main():
    tasks = [
        Agent().execute_async(f"Review {f}")
        for f in ["auth.py", "main.py", "utils.py"]
    ]
    results = await asyncio.gather(*tasks)
    for r in results:
        print(r.output)

asyncio.run(main())
```

### 7. Interactive REPL

```bash
# Terminal one-liner
nexus chat

# Output:
# Nexus Chat (type 'exit' to quit)
# > Fix bugs in auth.py
# [AI response streaming here]
# > exit
```

---

## Comparison with LangChain

| Aspect | LangChain | Nexus |
|--------|-----------|--------|
| **Install** | `pip install langchain` | `pip install nexus-ai` |
| **First agent** | 50 lines | 3 lines |
| **Agent creation** | `LLMChain(llm=..., prompt=...)` | `Agent()` |
| **Execution** | `chain.run("task")` | `agent("task")` |
| **Streaming** | `.stream()` | `.execute(stream=True)` |
| **Context manager** | ❌ | ✅ `with Agent() as a:` |
| **One-liners** | ❌ | ✅ `code_review("file.py")` |
| **Sandbox by default** | ❌ | ✅ |
| **Local models** | ⚠️ | ✅ |
| **Debug mode** | Hard to find | ✅ `.debug()` |
| **Type safety** | ⚠️ | ✅ Full |

---

## Installation & Setup

```bash
# One command
pip install nexus-ai

# That's it. No config, no API keys needed for local.
```

### Dependencies

```toml
# pyproject.toml (auto-installed)
dependencies = [
    "websockets>=12.0",
    "aiohttp>=3.9.0",  # For async HTTP fallback
]

# Optional (for Rust acceleration)
[project.optional-dependencies]
native = ["nexus-ai-native"]  # PyO3 extension

# For development
[project.optional-dependencies]
dev = [
    "pytest>=8.0",
    "pytest-asyncio>=0.23",
    "ruff>=0.2",
    "mypy>=1.8",
    "pre-commit>=3.0",
]
```

---

## File Structure

```
nexus-protocol/crates/nexus-sdk/python/
├── nexus/
│   ├── __init__.py          # Public API
│   ├── _native.py          # PyO3 bindings (auto-generated)
│   ├── _lazy.py            # Lazy evaluation
│   ├── _async.py           # Async utilities
│   └── _types.py           # Type definitions
├── tests/
│   ├── test_agent.py
│   ├── test_lazy.py
│   ├── test_streaming.py
│   └── test_integration.py
├── examples/
│   ├── quickstart.py       # 3-line demo
│   ├── code_review.py      # File operations
│   ├── streaming.py        # Token streaming
│   └── async_pipeline.py   # Parallel execution
├── pyproject.toml
├── README.md
└── Makefile               # pip install -e . && make test
```

---

## Quality Checklist

- [ ] `pip install nexus-ai` works in < 10 seconds
- [ ] `python -c "from nexus import Agent; print(Agent()('hello'))"` works
- [ ] All one-liners work: `code_review`, `fix_bugs`, `explain_code`
- [ ] Context manager: `with Agent() as a: a.execute("x")`
- [ ] Streaming: `for token in agent.execute("x", stream=True):`
- [ ] Error messages are helpful
- [ ] Type hints on everything
- [ ] Docstrings with examples on everything
- [ ] No Python exception leaks to user (they see NexusError)
- [ ] Async works: `await agent.execute_async()`
- [ ] 30-second demo actually works

---

## The Killer Demo (30 Seconds)

```bash
# STEP 1: Install (5 seconds)
pip install nexus-ai

# STEP 2: Run (10 seconds)
python << 'EOF'
from nexus import Agent
result = Agent()("Say hello")
print(result.output)
EOF

# STEP 3: Star! (2 seconds)
# https://github.com/vortex-ia/nexus-protocol
```

If this doesn't work in 30 seconds, we failed.
