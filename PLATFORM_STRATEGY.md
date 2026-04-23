# Nexus Platform Strategy — Last-Mile Infrastructure

**The Big Insight:** O protocolo e SDK não são um produto. São **plataforma** que reduz CAC de todos os outros projetos pra perto de zero.

---

## O Modelo de Plataforma

```
┌─────────────────────────────────────────────────────────────┐
│                      NEXUS PLATFORM                         │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              NEXUS PROTOCOL (Rust)                   │  │
│  │  - WASM Sandbox                                       │  │
│  │  - WebSocket duplex                                  │  │
│  │  - Ollama/Local LLMs                                 │  │
│  │  - Type-safe messages                                │  │
│  └─────────────────────────────────────────────────────┘  │
│                         │                                  │
│                         ▼                                  │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              NEXUS SDK (Rust + PyO3)                  │  │
│  │  - Polars/Dask lazy evaluation                        │  │
│  │  - 10x faster than LangChain                          │  │
│  │  - 3 lines para um agent                             │  │
│  └─────────────────────────────────────────────────────┘  │
│                         │                                  │
│           ┌────────────┼────────────┐                      │
│           ▼            ▼            ▼                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  VORTEX IDE  │  │  SOLVEOS    │  │  MEU NEXT   │   │
│  │  (your CLI)  │  │  (backend)  │  │  (whatever) │   │
│  │              │  │              │  │              │   │
│  │   Free       │  │  $20/mo      │  │  Monetize   │   │
│  │   Stars =     │  │  Revenue    │  │  with less  │   │
│  │   Marketing  │  │  generated  │  │  effort    │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                             │
│  CAC PRÓXIMO DE ZERO = protocolo faz marketing sozinho     │
└─────────────────────────────────────────────────────────────┘
```

---

## Why This Works

### Traditional CAC

```
Você constrói projeto A → gasta $100 em marketing → CAC = $100/cliente
Você constrói projeto B → gasta $100 em marketing → CAC = $100/cliente
Você constrói projeto C → gasta $100 em marketing → CAC = $100/cliente
```

### Platform CAC

```
Você constrói NEXUS PROTOCOL → comunidade gera stars + marketing orgânico
Projeto A usa Nexus → marketing orgânico do Nexus ajuda Projeto A
Projeto B usa Nexus → marketing orgânico do Nexus ajuda Projeto B
Projeto C usa Nexus → marketing orgânico do Nexus ajuda Projeto C

RESULTADO: CAC = $10/cliente (apenas manter protocolo)
```

---

## The Rust + Python Strategy

### Why Rust (Not Pure Python)

| Aspect | Python SDK | Rust + PyO3 SDK |
|--------|-----------|-----------------|
| **Performance** | 1x | 10-100x |
| **Startup time** | Slow | Instant |
| **Memory** | High | Low |
| **Binary size** | Big | Small |
| **GIL issues** | Yes | No |
| **Developer love** | Medium | HIGH |

### Polars/Dask Patterns (The Magic)

Polars transformou análise de dados em algo pythônico:

```python
# Polars: você descreve O QUE quer, não COMO fazer
df = (
    pl.read_csv("data.csv")
      .filter(pl.col("age") > 21)
      .with_columns([
          pl.col("name").str.to_uppercase(),
          pl.col("salary").mean().over("department"),
      ])
      .sort("salary", descending=True)
)

# Lazy mode: não executa até collect()
df = (
    pl.read_csv("data.csv")
      .filter(pl.col("age") > 21)
      .group_by("department")
      .agg([pl.col("salary").mean(), pl.col("*").count()])
)

# QUANDO vc faz .collect(), ele otimiza o graph inteiro
result = df.collect()  # Agora sim, executa otimizado
```

**Vamos fazer o MESMO pro código AI-execution:**

```python
# Nexus Lazy Agent (Polars-style)
agent = (
    NexusAgent(model="qwen2.5-coder:3b")
      .tools("filesystem", "terminal")
      .system("You are a senior developer")
      .timeout(30)
    # NADA executa aqui. É só descricao.
)

# Quando vc chama .execute(), ele otimiza o graph
result = agent.execute("Fix bugs in auth.py")

# Ou streaming
for token in agent.execute("Write tests", stream=True):
    print(token, end="", flush=True)
```

### O Computation Graph (Dask-style)

Dask divide tarefas grandes em 작은 partes:

```
Tarefa grande
     │
     ▼
┌─────────┐
│  Map    │ ──► Partition 1
│         │ ──► Partition 2
│         │ ──► Partition 3
└─────────┘
     │
     ▼
┌─────────┐
│  Reduce │ ◄─── Results
└─────────┘
```

**Vamos fazer o MESMO pro código execution:**

```python
# Dask-style: Divide任务 em pequenas partes
tasks = (
    agent.tasks([
        "Fix bug in auth.py",
        "Write test for auth.py",
        "Update docs for auth.py",
    ])
    .map(lambda: execute_parallel())  # Paralelo
    .reduce(lambda: compile_and_notify())  # Agrega
)

# Executa tudo otimizado
results = tasks.execute()
```

---

## Features Que Vão Gerar ESTRELAS

### 1. The 30-Second Demo (MÁXIMA Prioridade)

```bash
# Install: 5 segundos
pip install nexus-ai

# Run: 10 segundos
nexus run "Say hello"

# Output: 2 segundos
Hello! I'm a Nexus AI agent. How can I help you?

# STAR NOW!
```

Se isso não funcionar em 30 segundos, perdemos o usuário.

### 2. The "Polars of AI Agents" Brand

```
LangChain = "Confuso demais"
LangGraph = "Graph demais"
Nexus = "Faz sentido. Funciona."

FastAPI fez HTTP ficar fácil.
Polars fez DataFrame ficar fácil.
Nexus vai fazer AI Agent ficar fácil.
```

### 3. The "Local-First" Privacy Pitch

```python
# Seus dados NUNCA saem da sua máquina
agent = Agent(
    model="qwen2.5-coder:3b",
    sandbox="wasm",  # Executa LOCAL
    cloud=False,      # ZERO dados externos
)

# Resultado: privacy-first + cost-free inference
```

### 4. The Speed Comparison

```
Benchmark (Rust vs Python SDK):
- LangChain: 150ms overhead
- Nexus SDK (Rust): 0.5ms overhead
- Speedup: 300x

LangChain: "Processing your request..."
Nexus: "Done." (instant)
```

### 5. The "Built with Nexus" Ecosystem

```markdown
<!-- README de qualquer projeto que usa Nexus -->
[![Built with Nexus](https://img.shields.io/badge/Built%20with-Nexus-blue)](https://github.com/vortex-ia/nexus-protocol)

# Este projeto usa Nexus Protocol para execução de código AI
```

**Cada projeto que usa Nexus = marketing orgânico.**

### 6. The Demo Viral

```
nexus.run (web demo)
├── 3 linhas de código
├── Execute no browser
├── Compartilha resultado
└── "Wow, isso é Nexus?"
```

---

## SDK Architecture (Rust + PyO3)

### Directory Structure

```
nexus-protocol/
├── crates/
│   ├── nexus-protocol-core/      # Rust core (MIT)
│   ├── nexus-sdk/
│   │   ├── rust/                 # Rust SDK (PyO3 bindings)
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── agent.rs      # Lazy agent
│   │   │   │   ├── graph.rs      # Computation graph
│   │   │   │   ├── execute.rs     # Optimized execution
│   │   │   │   └── pyo3 mod.rs   # Python bindings
│   │   │   ├── Cargo.toml
│   │   │   └── pyproject.toml
│   │   ├── python/               # Python wrapper (thin)
│   │   └── typescript/
├── SPEC.md
├── SECURITY_AUDIT.md
├── STAR_STRATEGY.md
└── README.md
```

### Rust Core (High-Performance)

```rust
// nexus-sdk/rust/src/agent.rs

use pyo3::prelude::*;
use std::sync::Arc;

/// Lazy Agent — só constrói o graph, não executa
#[pyclass]
pub struct LazyAgent {
    model: String,
    tools: Vec<Tool>,
    system: Option<String>,
    timeout_ms: u64,
    config: AgentConfig,
}

#[pyclass]
pub struct ExecutionResult {
    #[pyo3(get)]
    pub output: String,
    #[pyo3(get)]
    pub tools_used: Vec<String>,
    #[pyo3(get)]
    pub duration_ms: u64,
    #[pyo3(get)]
    pub tokens_used: u32,
}

#[pymethods]
impl LazyAgent {
    fn tools(mut py: Py<Self>, names: Vec<String>) -> Py<Self> {
        // Builder pattern
        self.tools.extend(names.into_iter().map(|n| Tool::new(n)));
        Py::clone_ref(&py, self)
    }

    fn system(mut py: Py<Self>, prompt: String) -> Py<Self> {
        self.system = Some(prompt);
        Py::clone_ref(&py, self)
    }

    fn timeout(mut py: Py<Self>, ms: u64) -> Py<Self> {
        self.timeout_ms = ms;
        Py::clone_ref(&py, self)
    }

    /// QUANDO .execute() é chamado, o graph é otimizado E executado
    fn execute(&self, py: Python<'_>, prompt: String) -> PyResult<ExecutionResult> {
        // Bloqueia o GIL durante execução Rust (PyO3 pattern)
        let result = self.execute_internal(&prompt)?;
        Ok(result.into_py(py))
    }

    /// Async streaming (libpython compatible)
    fn execute_streaming(&self, py: Python<'_>, prompt: String) -> PyResult<Py<PyIterator>> {
        let gil = GILProtection::new();
        let stream = self.execute_stream_internal(&prompt)?;
        Ok(stream.into_py(py))
    }
}

/// Computation Graph (Dask-style)
#[pyclass]
pub struct TaskGraph {
    nodes: Vec<Node>,
    edges: Vec<(usize, usize)>,
    optimized: bool,
}

#[pyclass]
impl TaskGraph {
    fn map(&mut self, tasks: Vec<String>) -> &mut Self {
        // Divide tasks em pequenas partes
        self.nodes.extend(tasks.into_iter().map(Node::new_task));
        self
    }

    fn reduce(&mut self, aggregator: String) -> &mut Self {
        self.nodes.push(Node::new_reducer(aggregator));
        self
    }

    /// Otimiza o graph ANTES de executar
    fn optimize(&mut self) -> &mut Self {
        // 1. Remove tarefas redundantes
        // 2. Fusão de tarefas adjacentes
        // 3. Reordena para mínimo de rede
        self.optimized = true;
        self
    }

    /// Executa o graph otimizado
    fn execute(&self) -> PyResult<Vec<ExecutionResult>> {
        if !self.optimized {
            self.optimize();
        }
        self.execute_internal()
    }
}
```

### Python Wrapper (Thin, Ergonomic)

```python
# nexus_sdk/python/nexus.py (thin wrapper)
"""
Nexus AI SDK — Polars-style API for AI Agents

Example:
    >>> import nexus
    >>> agent = nexus.Agent(model="qwen2.5-coder:3b")
    >>> result = agent.execute("Say hello")
    >>> print(result.output)
"""

from __future__ import annotations

from typing import Optional, Iterator
from dataclasses import dataclass

# Import from Rust extension (.so file)
from nexus_sdk._native import (
    LazyAgent as _LazyAgent,
    TaskGraph as _TaskGraph,
    ExecutionResult as _ExecutionResult,
    Tool as _Tool,
)


class Agent:
    """
    Polars-style lazy agent for AI code execution.

    Usage:
        >>> agent = nexus.Agent(model="qwen2.5-coder:3b")
        >>> result = agent.execute("Fix bugs in auth.py")

        >>> # Streaming
        >>> for token in agent.execute("Write code", stream=True):
        ...     print(token, end="")

        >>> # Lazy (não executa até .collect())
        >>> agent = nexus.Agent().tools("filesystem").timeout(30)
        >>> # ... mais nada
    """

    def __init__(
        self,
        model: str = "qwen2.5-coder:3b",
        *,
        tools: Optional[list[str]] = None,
        system: Optional[str] = None,
        timeout: int = 30,
    ):
        self._agent = _LazyAgent(model=model)
        if tools:
            self._agent = self._agent.tools(tools)
        if system:
            self._agent = self._agent.system(system)
        if timeout:
            self._agent = self._agent.timeout(timeout * 1000)

    def execute(self, prompt: str, *, stream: bool = False) -> ExecutionResult | Iterator[str]:
        """Execute the agent. Returns result or token stream."""
        if stream:
            return self._agent.execute_streaming(prompt)
        return self._agent.execute(prompt)

    def __repr__(self) -> str:
        return f"Agent(model={self._agent.model})"


@dataclass
class ExecutionResult:
    """Result from agent execution."""
    output: str
    tools_used: list[str]
    duration_ms: int
    tokens_used: int


class TaskGraph:
    """
    Dask-style computation graph for parallel execution.

    Usage:
        >>> graph = (
        ...     nexus.TaskGraph()
        ...     .tasks([
        ...         "Fix bug in auth.py",
        ...         "Write tests for auth.py",
        ...     ])
        ...     .execute()
        ... )
    """

    def __init__(self):
        self._graph = _TaskGraph()

    def tasks(self, task_list: list[str]) -> TaskGraph:
        """Add tasks to the graph."""
        self._graph = self._graph.map(task_list)
        return self

    def reduce(self, reducer: str) -> TaskGraph:
        """Add a reducer."""
        self._graph = self._graph.reduce(reducer)
        return self

    def optimize(self) -> TaskGraph:
        """Optimize the graph (Polars-style)."""
        self._graph.optimize()
        return self

    def execute(self) -> list[ExecutionResult]:
        """Execute the optimized graph."""
        return self._graph.execute()


# Aliases for Polars-style naming
LazyAgent = Agent
```

---

## Protocol Ergonomics Improvements

### Current Issues

```python
# Current: Verbose, multi-step
client = NexusClient("ws://localhost:8080")
await client.connect()
result = await client.execute(code, language="python")
await client.close()
```

### Improved: Polars-style

```python
# Improved: Chainable, auto-connect
result = (
    NexusClient()
    .connect()  # or just: .execute() auto-connects
    .execute("print('hello')", language="python")
    .close()  # or: use context manager
)
```

### Context Manager (Best)

```python
# Best: Context manager (like Polars)
async with Nexus() as client:
    result = await client.execute("print('hello')", language="python")

# Inside __aexit__, client.close() is called automatically
```

### Pipeline Operator

```python
# Pipeline (like Unix pipes)
result = (
    Nexus()
    | "print('hello')"  # send message
    | Language.PYTHON    # set language
    | SandboxPolicy.DEFAULT  # set policy
    | Execute()           # execute
    | Await()            # wait for result
)
```

---

## Star Generation Machine

### The 10x Viral Loop

```
1. Dev encontra Nexus via:
   - "Show HN: Nexus Protocol"
   - Twitter thread viral
   - YouTube tutorial

2. Vê o demo (30 segundos):
   pip install nexus-ai && nexus run "Say hello"

3. Fica impressionado:
   - "Wow, isso é rápido"
   - "Wow, isso é fácil"
   - "Wow, isso é seguro"

4. Conta pra amigos:
   - Twitter: "Acabei de descobrir @nexus_protocol..."
   - Discord: "Cara, usa isso"
   - WhatsApp: "Manda o link"

5. Volta e estrela:
   - ⭐⭐⭐⭐⭐
   - Posta no LinkedIn
   - Gera badge

6. Vira contributor:
   - PR aceito = badge
   - Badge = social proof
   - Social proof = mais contributors
```

### Content Calendar

```
Week 1: "Why MCP Sucks and Nexus is Better"
Week 2: "Nexus vs LangChain: A Benchmark"
Week 3: YouTube Tutorial (5 min)
Week 4: "Show HN: We Built a FastAPI for AI Agents"
Week 5: Case study (early adopter)
Week 6: "How We Got 1000 Stars in 30 Days"
Repete...
```

---

## Summary

### The Platform Flywheel

```
┌────────────────────────────────────────┐
│  NEXUS PROTOCOL (Open Source)          │
│  - Gera stars e credibilidade          │
│  - Gera developers contributors       │
│  - Gera buzz e marketing orgânico      │
└─────────────────┬──────────────────────┘
                  │
                  ▼
┌────────────────────────────────────────┐
│  NEXUS SDK (Rust + PyO3)               │
│  - Faz todos os projetos serem mais   │
│    competitivos (performance + DX)      │
└─────────────────┬──────────────────────┘
                  │
                  ▼
┌────────────────────────────────────────┐
│  SEUS PROJETOS (CAC ~ Zero)            │
│  - Vortex IDE (CLI)                   │
│  - SolveOS (backend)                  │
│  - Meu próximo projeto                │
│  - + todos que usarem Nexus           │
└────────────────────────────────────────┘
```

### O Que Construir (Prioridade)

1. **Rust SDK com PyO3** — performance 10x
2. **Polars/Dask lazy patterns** — ergonomia
3. **30-second demo** — viral loop
4. **Protocol context manager** — ergonomia
5. **"Built with Nexus" badges** — ecosystem

### Métricas de Sucesso

| Métrica | 30 dias | 60 dias | 90 dias |
|---------|---------|---------|---------|
| GitHub Stars | 100 | 1,000 | 10,000 |
| PyPI downloads | 1,000 | 10,000 | 100,000 |
| Projects using Nexus | 5 | 50 | 200 |
| Stars/project | - | - | - |
| **CAC reduction** | 20% | 50% | **~0%** |

---

**Key Insight:**
> Não estou vendendo um produto. Estou construindo infraestrutura que faz TODOS os meus projetos mais competitivos, e o marketing do Nexus ajuda cada projeto automaticamente.
