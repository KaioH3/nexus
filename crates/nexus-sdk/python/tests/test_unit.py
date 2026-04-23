"""
Unit Tests — Isolated component tests.

Run with: pytest tests/test_unit.py -v
"""

import asyncio
import pytest
from unittest.mock import Mock, patch, AsyncMock
from typing import Any

from nexus_protocol import (
    Agent,
    AgentConfig,
    ExecutionResult,
    SandboxPolicy,
    Memory,
    Message,
    Tool,
    FilesystemTool,
    TerminalTool,
    ErrorCode,
    NexusError,
    lazy,
    LazyAgent,
)
from nexus_protocol.providers import (
    Provider,
    GroqProvider,
    OllamaProvider,
    GenerationOptions,
    GenerationResult,
)


# ============================================================================
# AgentConfig Tests
# ============================================================================

@pytest.mark.unit
class TestAgentConfig:
    """Test AgentConfig defaults and options."""

    def test_config_defaults(self):
        """Default configuration."""
        config = AgentConfig()
        assert config.strategy == "react"
        assert config.temperature == 0.7
        assert config.max_tokens == 4096
        assert config.retry_attempts == 3
        assert config.timeout_ms == 30000

    def test_config_custom(self):
        """Custom configuration."""
        config = AgentConfig(
            strategy="cot",
            temperature=0.3,
            max_tokens=2048,
            retry_attempts=5,
            timeout_ms=60000,
        )
        assert config.strategy == "cot"
        assert config.temperature == 0.3
        assert config.max_tokens == 2048
        assert config.retry_attempts == 5
        assert config.timeout_ms == 60000

    def test_config_debug_flag(self):
        """Debug flag configuration."""
        config = AgentConfig(debug=True)
        assert config.debug is True

    def test_config_system_prompt(self):
        """Custom system prompt."""
        config = AgentConfig(custom_system_prompt="You are a pirate")
        assert config.custom_system_prompt == "You are a pirate"


# ============================================================================
# Memory Tests
# ============================================================================

@pytest.mark.unit
class TestMemory:
    """Test Memory class."""

    def test_memory_empty_init(self):
        """Empty memory initialization."""
        memory = Memory()
        assert len(memory.messages) == 0
        assert memory.summary is None

    def test_memory_add_message(self):
        """Add message to memory."""
        memory = Memory()
        memory.add("user", "Hello")
        assert len(memory.messages) == 1
        assert memory.messages[0].role == "user"
        assert memory.messages[0].content == "Hello"

    def test_memory_get_context_empty(self):
        """Get context from empty memory."""
        memory = Memory()
        context = memory.get_context()
        assert context == ""

    def test_memory_get_context_with_messages(self):
        """Get context with messages."""
        memory = Memory()
        memory.add("user", "Hello")
        memory.add("assistant", "Hi there")
        context = memory.get_context()
        assert "user: Hello" in context
        assert "assistant: Hi there" in context

    def test_memory_clear(self):
        """Clear memory."""
        memory = Memory()
        memory.add("user", "Hello")
        memory.clear()
        assert len(memory.messages) == 0
        assert memory.summary is None

    def test_memory_summarization_triggers(self):
        """Summarization triggers when exceeding max_tokens."""
        memory = Memory(max_tokens=10)
        # Add a long message (simulated)
        memory.add("user", "a" * 100)
        memory.add("user", "b" * 100)
        # Should trigger summarization
        context = memory.get_context()
        assert memory.summary is not None or len(context) < 200


# ============================================================================
# Tool Tests
# ============================================================================

@pytest.mark.unit
class TestTools:
    """Test Tool classes."""

    def test_filesystem_tool_creation(self):
        """FilesystemTool can be created."""
        tool = FilesystemTool(read=True, write=False)
        assert tool.name == "filesystem"
        assert tool.config["read"] is True
        assert tool.config["write"] is False

    def test_filesystem_tool_allowed_paths(self):
        """FilesystemTool with allowed paths."""
        tool = FilesystemTool(allowed_paths=["/tmp", "/home"])
        assert "/tmp" in tool.allowed_paths
        assert "/home" in tool.allowed_paths

    def test_terminal_tool_creation(self):
        """TerminalTool can be created."""
        tool = TerminalTool(allowed_commands=["git", "ls"], timeout_seconds=60)
        assert tool.name == "terminal"
        assert "git" in tool.config["allowed_commands"]
        assert tool.config["timeout"] == 60

    def test_tool_to_dict(self):
        """Tool can be serialized to dict."""
        tool = FilesystemTool()
        d = tool.to_dict()
        assert d["name"] == "filesystem"
        assert isinstance(d["config"], dict)


# ============================================================================
# LazyAgent Tests
# ============================================================================

@pytest.mark.unit
class TestLazyAgent:
    """Test LazyAgent class."""

    def test_lazy_agent_creation(self):
        """LazyAgent can be created."""
        agent = LazyAgent(model="test-model")
        assert agent._model == "test-model"

    def test_lazy_agent_tools(self):
        """LazyAgent tools method."""
        agent = LazyAgent()
        result = agent.tools("filesystem", "terminal")
        assert result is agent
        assert "filesystem" in agent._tools
        assert "terminal" in agent._tools

    def test_lazy_agent_system(self):
        """LazyAgent system method."""
        agent = LazyAgent()
        result = agent.system("You are helpful")
        assert result is agent
        assert agent._config.get("system_prompt") == "You are helpful"

    def test_lazy_agent_timeout(self):
        """LazyAgent timeout method."""
        agent = LazyAgent()
        result = agent.timeout(60)
        assert result is agent
        assert agent._config.get("timeout_ms") == 60000

    def test_lazy_function(self):
        """lazy() helper function."""
        agent = lazy(model="test-model")
        assert isinstance(agent, LazyAgent)
        assert agent._model == "test-model"

    def test_lazy_with_tools(self):
        """lazy() with tools."""
        agent = lazy(tools=["filesystem"])
        assert "filesystem" in agent._tools


# ============================================================================
# ExecutionResult Tests
# ============================================================================

@pytest.mark.unit
class TestExecutionResult:
    """Test ExecutionResult class."""

    def test_result_with_all_fields(self):
        """Result with all fields."""
        result = ExecutionResult(
            output="Hello",
            tools_used=["tool1", "tool2"],
            duration_ms=100,
            tokens_used=50,
            cache_hit=True,
        )
        assert result.output == "Hello"
        assert len(result.tools_used) == 2
        assert result.duration_ms == 100
        assert result.tokens_used == 50
        assert result.cache_hit is True

    def test_result_ok_property(self):
        """Result ok property."""
        result_with_output = ExecutionResult(output="test")
        result_empty = ExecutionResult(output="")

        assert result_with_output.ok is True
        assert result_empty.ok is False

    def test_result_str(self):
        """Result string representation."""
        result = ExecutionResult(output="test output")
        assert str(result) == "test output"


# ============================================================================
# NexusError Tests
# ============================================================================

@pytest.mark.unit
class TestNexusError:
    """Test NexusError class."""

    def test_error_basic(self):
        """Basic error."""
        err = NexusError(ErrorCode.COMPILATION_FAILED, "rustc error")
        assert err.code == ErrorCode.COMPILATION_FAILED
        assert err.message == "rustc error"
        assert err.request_id is None

    def test_error_with_request_id(self):
        """Error with request ID."""
        err = NexusError(ErrorCode.TIMEOUT, "operation timed out", "req-456")
        assert err.request_id == "req-456"

    def test_error_string_format(self):
        """Error string formatting."""
        err = NexusError(ErrorCode.RATE_LIMITED, "too many requests")
        s = str(err)
        assert "rate_limited" in s
        assert "too many requests" in s

    def test_error_string_with_request_id(self):
        """Error string with request ID."""
        err = NexusError(ErrorCode.INTERNAL_ERROR, "crash", "req-789")
        s = str(err)
        assert "req-789" in s

    def test_all_error_codes_exist(self):
        """All error codes are defined."""
        expected_codes = [
            "handshake_failed",
            "version_mismatch",
            "compilation_failed",
            "sandbox_violation",
            "sandbox_timeout",
            "sandbox_out_of_memory",
            "ollama_connection_failed",
            "ollama_generation_failed",
            "network_blocked",
            "syscall_blocked",
            "file_not_found",
            "permission_denied",
            "invalid_message",
            "missing_api_key",
            "invalid_api_key",
            "rate_limited",
            "internal_error",
        ]
        for code_str in expected_codes:
            assert hasattr(ErrorCode, code_str.upper())
            assert ErrorCode[code_str.upper()].value == code_str


# ============================================================================
# SandboxPolicy Tests
# ============================================================================

@pytest.mark.unit
class TestSandboxPolicy:
    """Test SandboxPolicy class."""

    def test_default_policy(self):
        """Default policy."""
        policy = SandboxPolicy()
        assert policy.max_memory_mb == 512
        assert policy.max_cpu_time_ms == 30000
        assert policy.allowed_network is False

    def test_zero_trust_policy(self):
        """Zero trust policy is most restrictive."""
        policy = SandboxPolicy.zero_trust()
        assert policy.max_memory_mb == 128
        assert policy.max_cpu_time_ms == 5000
        assert policy.allowed_network is False
        assert policy.allowed_paths == []
        assert len(policy.blocked_syscalls) > 10

    def test_ai_generated_policy(self):
        """AI generated code policy."""
        policy = SandboxPolicy.ai_generated_code()
        assert policy.max_memory_mb == 512
        assert policy.allowed_network is False
        assert "/tmp" in policy.allowed_paths

    def test_development_policy(self):
        """Development policy is most permissive."""
        policy = SandboxPolicy.development()
        assert policy.max_memory_mb == 1024
        assert policy.allowed_network is True
        assert "/workspace" in policy.allowed_paths

    def test_policy_to_dict(self):
        """Policy serialization."""
        policy = SandboxPolicy.development()
        d = policy.to_dict()
        assert isinstance(d, dict)
        assert d["max_memory_mb"] == 1024
        assert d["allowed_network"] is True
        assert isinstance(d["blocked_syscalls"], list)


# ============================================================================
# Provider Interface Tests
# ============================================================================

@pytest.mark.unit
class TestProviderInterface:
    """Test provider interface."""

    def test_provider_is_base_class(self):
        """Provider is a base class."""
        assert Provider is not None

    def test_groq_provider_inheritance(self):
        """GroqProvider inherits from Provider."""
        assert issubclass(GroqProvider, Provider)

    def test_ollama_provider_inheritance(self):
        """OllamaProvider inherits from Provider."""
        assert issubclass(OllamaProvider, Provider)


# ============================================================================
# GenerationOptions Tests
# ============================================================================

@pytest.mark.unit
class TestGenerationOptions:
    """Test GenerationOptions class."""

    def test_default_options(self):
        """Default generation options."""
        opts = GenerationOptions()
        assert opts.temperature == 0.7
        assert opts.max_tokens == 1024
        assert opts.top_p == 0.9
        assert opts.top_k == 40
        assert opts.stop is None

    def test_custom_options(self):
        """Custom generation options."""
        opts = GenerationOptions(
            temperature=0.5,
            max_tokens=500,
            top_p=0.8,
            top_k=20,
            stop=["END", "STOP"],
        )
        assert opts.temperature == 0.5
        assert opts.max_tokens == 500
        assert opts.top_p == 0.8
        assert opts.top_k == 20
        assert opts.stop == ["END", "STOP"]


# ============================================================================
# GenerationResult Tests
# ============================================================================

@pytest.mark.unit
class TestGenerationResult:
    """Test GenerationResult class."""

    def test_result_creation(self):
        """GenerationResult can be created."""
        result = GenerationResult(
            output="Generated text",
            model="llama-3.3-70b-versatile",
            prompt_tokens=10,
            completion_tokens=20,
            total_tokens=30,
            duration_ms=1500,
        )
        assert result.output == "Generated text"
        assert result.model == "llama-3.3-70b-versatile"
        assert result.prompt_tokens == 10
        assert result.completion_tokens == 20
        assert result.total_tokens == 30
        assert result.duration_ms == 1500

    def test_result_str(self):
        """GenerationResult string representation."""
        result = GenerationResult(output="test", model="m", prompt_tokens=1, completion_tokens=1, total_tokens=2, duration_ms=1)
        assert str(result) == "test"


# ============================================================================
# Run All Unit Tests
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
