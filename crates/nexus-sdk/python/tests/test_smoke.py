"""
Smoke Tests — Quick sanity checks to verify basic functionality.

Run with: pytest tests/test_smoke.py -v

These tests should complete in < 30 seconds total.
"""

import pytest

from nexus_protocol import (
    Agent,
    SyncAgent,
    ExecutionResult,
    SandboxPolicy,
    ErrorCode,
    NexusError,
    code_review,
    fix_bugs,
    explain_code,
    generate_tests,
    run_terminal,
)


# ============================================================================
# Import Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestImports:
    """Verify all imports work correctly."""

    def test_import_agent(self):
        """Agent can be imported."""
        from nexus_protocol import Agent
        assert Agent is not None

    def test_import_execution_result(self):
        """ExecutionResult can be imported."""
        from nexus_protocol import ExecutionResult
        assert ExecutionResult is not None

    def test_import_sandbox_policy(self):
        """SandboxPolicy can be imported."""
        from nexus_protocol import SandboxPolicy
        assert SandboxPolicy is not None

    def test_import_error_types(self):
        """Error types can be imported."""
        from nexus_protocol import ErrorCode, NexusError
        assert ErrorCode is not None
        assert NexusError is not None

    def test_import_one_liners(self):
        """One-liner functions can be imported."""
        from nexus_protocol import code_review, fix_bugs, explain_code
        assert callable(code_review)
        assert callable(fix_bugs)
        assert callable(explain_code)

    def test_import_providers(self):
        """Providers can be imported."""
        from nexus_protocol.providers import GroqProvider, OllamaProvider
        assert GroqProvider is not None
        assert OllamaProvider is not None


# ============================================================================
# Constructor Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestAgentConstruction:
    """Verify Agent can be constructed in various ways."""

    def test_agent_default_constructor(self):
        """Agent with defaults."""
        agent = Agent()
        assert agent is not None
        assert agent._model == "qwen2.5-coder:3b"

    def test_agent_with_model(self):
        """Agent with custom model."""
        agent = Agent(model="llama-3.2")
        assert agent._model == "llama-3.2"

    def test_agent_with_tools(self):
        """Agent with tools."""
        agent = Agent(tools=["filesystem", "terminal"])
        assert "filesystem" in agent._tools
        assert "terminal" in agent._tools

    def test_agent_builder_pattern(self):
        """Agent with builder pattern."""
        agent = (
            Agent()
            .tools("filesystem")
            .system("You are helpful")
            .timeout(30)
        )
        assert "filesystem" in agent._tools
        assert agent._config.custom_system_prompt == "You are helpful"
        assert agent._config.timeout_ms == 30000

    def test_sync_agent(self):
        """SyncAgent can be constructed."""
        agent = SyncAgent()
        assert agent is not None


# ============================================================================
# Sandbox Policy Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestSandboxPolicy:
    """Verify SandboxPolicy works correctly."""

    def test_sandbox_policy_default(self):
        """Default sandbox policy."""
        policy = SandboxPolicy()
        assert policy.max_memory_mb == 512
        assert policy.max_cpu_time_ms == 30000

    def test_sandbox_policy_zero_trust(self):
        """Zero trust policy."""
        policy = SandboxPolicy.zero_trust()
        assert policy.max_memory_mb == 128
        assert policy.allowed_network is False
        assert policy.allowed_paths == []

    def test_sandbox_policy_ai_generated(self):
        """AI generated code policy."""
        policy = SandboxPolicy.ai_generated_code()
        assert policy.max_memory_mb == 512
        assert policy.allowed_network is False

    def test_sandbox_policy_development(self):
        """Development policy."""
        policy = SandboxPolicy.development()
        assert policy.allowed_network is True
        assert policy.max_memory_mb == 1024

    def test_sandbox_policy_to_dict(self):
        """Policy can be serialized to dict."""
        policy = SandboxPolicy.ai_generated_code()
        d = policy.to_dict()
        assert isinstance(d, dict)
        assert "max_memory_mb" in d
        assert "blocked_syscalls" in d


# ============================================================================
# Error Types Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestErrorTypes:
    """Verify error handling works correctly."""

    def test_error_code_values(self):
        """ErrorCode enum has expected values."""
        assert ErrorCode.HANDSHAKE_FAILED.value == "handshake_failed"
        assert ErrorCode.SANDBOX_VIOLATION.value == "sandbox_violation"
        assert ErrorCode.RATE_LIMITED.value == "rate_limited"

    def test_nexus_error_construction(self):
        """NexusError can be constructed."""
        err = NexusError(ErrorCode.COMPILATION_FAILED, "rustc failed")
        assert err.code == ErrorCode.COMPILATION_FAILED
        assert err.message == "rustc failed"

    def test_nexus_error_with_request_id(self):
        """NexusError with request ID."""
        err = NexusError(ErrorCode.TIMEOUT, "took too long", "req-123")
        assert err.request_id == "req-123"
        assert "req-123" in str(err)

    def test_nexus_error_string_representation(self):
        """NexusError has good string representation."""
        err = NexusError(ErrorCode.INTERNAL_ERROR, "Something broke")
        s = str(err)
        assert "internal_error" in s
        assert "Something broke" in s


# ============================================================================
# ExecutionResult Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestExecutionResult:
    """Verify ExecutionResult works correctly."""

    def test_execution_result_creation(self):
        """ExecutionResult can be created."""
        result = ExecutionResult(
            output="Hello, World!",
            tools_used=["filesystem"],
            duration_ms=1500,
            tokens_used=50,
        )
        assert result.output == "Hello, World!"
        assert "filesystem" in result.tools_used
        assert result.duration_ms == 1500
        assert result.tokens_used == 50

    def test_execution_result_str(self):
        """ExecutionResult has string representation."""
        result = ExecutionResult(output="test output")
        assert str(result) == "test output"

    def test_execution_result_bool(self):
        """ExecutionResult has boolean interpretation."""
        result_ok = ExecutionResult(output="test")
        result_empty = ExecutionResult(output="")

        assert result_ok.ok is True
        assert result_empty.ok is False

    def test_execution_result_repr(self):
        """ExecutionResult has repr."""
        result = ExecutionResult(output="Hello, World! How are you today?")
        r = repr(result)
        assert "Hello, World!" in r


# ============================================================================
# One-Liner Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestOneLinerSignatures:
    """Verify one-liner functions have correct signatures."""

    def test_code_review_signature(self):
        """code_review takes path."""
        import inspect
        sig = inspect.signature(code_review)
        assert "path" in sig.parameters

    def test_fix_bugs_signature(self):
        """fix_bugs takes path."""
        import inspect
        sig = inspect.signature(fix_bugs)
        assert "path" in sig.parameters

    def test_explain_code_signature(self):
        """explain_code takes code."""
        import inspect
        sig = inspect.signature(explain_code)
        assert "code" in sig.parameters

    def test_generate_tests_signature(self):
        """generate_tests takes path."""
        import inspect
        sig = inspect.signature(generate_tests)
        assert "path" in sig.parameters


# ============================================================================
# Agent Builder Pattern Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestAgentBuilderPattern:
    """Verify builder pattern is chainable."""

    def test_tools_chainable(self):
        """tools() returns self."""
        agent = Agent()
        result = agent.tools("filesystem")
        assert result is agent

    def test_system_chainable(self):
        """system() returns self."""
        agent = Agent()
        result = agent.system("You are helpful")
        assert result is agent
        assert agent._config.custom_system_prompt == "You are helpful"

    def test_timeout_chainable(self):
        """timeout() returns self."""
        agent = Agent()
        result = agent.timeout(60)
        assert result is agent
        assert agent._config.timeout_ms == 60000

    def test_retry_chainable(self):
        """retry() returns self."""
        agent = Agent()
        result = agent.retry(5, 2.0)
        assert result is agent
        assert agent._config.retry_attempts == 5
        assert agent._config.retry_delay == 2.0

    def test_temperature_chainable(self):
        """temperature() returns self."""
        agent = Agent()
        result = agent.temperature(0.5)
        assert result is agent
        assert agent._config.temperature == 0.5

    def test_full_chain(self):
        """Full builder chain works."""
        agent = (
            Agent()
            .tools("filesystem", "terminal")
            .system("You are an expert")
            .timeout(30)
            .temperature(0.7)
            .retry(3, 1.0)
        )
        assert "filesystem" in agent._tools
        assert "terminal" in agent._tools
        assert agent._config.custom_system_prompt == "You are an expert"
        assert agent._config.timeout_ms == 30000
        assert agent._config.temperature == 0.7
        assert agent._config.retry_attempts == 3


# ============================================================================
# Provider Smoke Tests
# ============================================================================

@pytest.mark.smoke
class TestProviders:
    """Verify providers can be constructed."""

    def test_groq_provider_requires_key(self):
        """GroqProvider requires API key."""
        from nexus_protocol.providers import GroqProvider
        with pytest.raises(ValueError, match="API key"):
            GroqProvider(api_key="")

    def test_groq_provider_with_key(self):
        """GroqProvider works with valid key."""
        from nexus_protocol.providers import GroqProvider
        provider = GroqProvider(api_key="test_key_123")
        assert provider.api_key == "test_key_123"

    def test_ollama_provider_default_url(self):
        """OllamaProvider has default URL."""
        from nexus_protocol.providers import OllamaProvider
        provider = OllamaProvider()
        assert provider.base_url == "http://localhost:11434"

    def test_ollama_provider_custom_url(self):
        """OllamaProvider works with custom URL."""
        from nexus_protocol.providers import OllamaProvider
        provider = OllamaProvider(base_url="http://custom:11434")
        assert provider.base_url == "http://custom:11434"


# ============================================================================
# Run All Smoke Tests
# ============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
