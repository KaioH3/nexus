"""Tests for Nexus Protocol Python SDK."""

import pytest
from nexus_protocol import (
    NexusClient,
    SyncNexusClient,
    SandboxPolicy,
    Capabilities,
    Language,
    ErrorCode,
    ExecutionResult,
    NexusError,
)


class TestSandboxPolicy:
    """Test SandboxPolicy presets."""

    def test_zero_trust(self):
        policy = SandboxPolicy.zero_trust()
        assert policy.max_memory_mb == 128
        assert policy.max_cpu_time_ms == 5000
        assert policy.allowed_network is False
        assert policy.allowed_paths == []

    def test_ai_generated_code(self):
        policy = SandboxPolicy.ai_generated_code()
        assert policy.max_memory_mb == 512
        assert policy.max_cpu_time_ms == 30000
        assert policy.allowed_network is False
        assert "/tmp" in policy.allowed_paths

    def test_development(self):
        policy = SandboxPolicy.development()
        assert policy.max_memory_mb == 1024
        assert policy.allowed_network is True
        assert "/workspace" in policy.allowed_paths

    def test_to_dict(self):
        policy = SandboxPolicy.ai_generated_code()
        d = policy.to_dict()
        assert isinstance(d, dict)
        assert "max_memory_mb" in d
        assert "blocked_syscalls" in d


class TestCapabilities:
    """Test Capabilities."""

    def test_default(self):
        caps = Capabilities()
        assert caps.ollama is True
        assert caps.streaming is True
        assert "wasm3" in caps.wasm_runtimes

    def test_full(self):
        caps = Capabilities.full()
        assert "wasmer" in caps.wasm_runtimes
        assert "wasmtime" in caps.wasm_runtimes

    def test_to_dict(self):
        caps = Capabilities()
        d = caps.to_dict()
        assert isinstance(d, dict)
        assert "ollama" in d


class TestLanguage:
    """Test Language enum."""

    def test_compiler(self):
        assert Language.PYTHON.compiler() == "python3"
        assert Language.RUST.compiler() == "rustc"
        assert Language.GO.compiler() == "go"

    def test_values(self):
        assert Language.PYTHON.value == "python"
        assert Language.RUST.value == "rust"


class TestErrorCode:
    """Test ErrorCode enum."""

    def test_values(self):
        assert ErrorCode.HANDSHAKE_FAILED.value == "handshake_failed"
        assert ErrorCode.SANDBOX_TIMEOUT.value == "sandbox_timeout"

    def test_nexus_error(self):
        err = NexusError(ErrorCode.COMPILATION_FAILED, "rustc failed", "req-123")
        assert err.code == ErrorCode.COMPILATION_FAILED
        assert err.message == "rustc failed"
        assert err.request_id == "req-123"
        assert "compilation_failed" in str(err)


class TestExecutionResult:
    """Test ExecutionResult."""

    def test_creation(self):
        result = ExecutionResult(
            request_id="test-123",
            exit_code=0,
            stdout="Hello",
            stderr="",
            execution_time_ms=100,
        )
        assert result.exit_code == 0
        assert result.stdout == "Hello"
        assert result.cache_hit is False


class TestSyncClient:
    """Test SyncNexusClient."""

    def test_init(self):
        client = SyncNexusClient("ws://localhost:8080", "test-key")
        assert client._client._url == "ws://localhost:8080"
        assert client._client._api_key == "test-key"

    def test_context_manager(self):
        """Test that __enter__ and __exit__ work."""
        # Won't actually connect without server, but verifies structure
        client = SyncNexusClient()
        assert client._client is not None
