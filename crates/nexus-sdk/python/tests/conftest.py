"""
Pytest configuration and fixtures.
"""

import asyncio
import os
import pytest
from typing import Generator

from nexus_protocol import Agent, ExecutionResult
from nexus_protocol.providers import GroqProvider, OllamaProvider


# ============================================================================
# Configuration
# ============================================================================

GROQ_API_KEY = os.environ.get("GROQ_API_KEY")

OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://localhost:11434")

ENABLE_SLOW_TESTS = os.environ.get("ENABLE_SLOW_TESTS", "false").lower() == "true"
ENABLE_EXTERNAL_API = os.environ.get("ENABLE_EXTERNAL_API", "true").lower() == "true"


# ============================================================================
# Fixtures
# ============================================================================

@pytest.fixture(scope="session")
def event_loop() -> Generator:
    """Create session-scoped event loop."""
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    yield loop
    loop.close()


@pytest.fixture
def groq_api_key() -> str:
    """Get Groq API key from environment."""
    if not ENABLE_EXTERNAL_API:
        pytest.skip("External API tests disabled")
    return GROQ_API_KEY


@pytest.fixture
def ollama_url() -> str:
    """Get Ollama URL from environment."""
    return OLLAMA_URL


@pytest.fixture
async def groq_provider(groq_api_key) -> Generator:
    """Create Groq provider instance."""
    provider = GroqProvider(api_key=groq_api_key)
    yield provider
    await provider.close()


@pytest.fixture
async def ollama_provider(ollama_url) -> Generator:
    """Create Ollama provider instance."""
    provider = OllamaProvider(base_url=ollama_url)
    yield provider
    await provider.close()


@pytest.fixture
def agent() -> Agent:
    """Create Nexus agent with default settings."""
    return Agent(model="qwen2.5-coder:3b")


@pytest.fixture
def agent_with_groq(groq_api_key) -> Agent:
    """Create Nexus agent configured to use Groq."""
    return Agent(model="llama-3.3-70b-versatile", api_key=groq_api_key)


@pytest.fixture
def sync_agent() -> "SyncAgent":
    """Create synchronous agent."""
    from nexus_protocol import SyncAgent
    return SyncAgent(model="qwen2.5-coder:3b")


# ============================================================================
# Async Helpers
# ============================================================================

@pytest.fixture
async def async_agent():
    """Create async agent."""
    return Agent(model="qwen2.5-coder:3b")


# ============================================================================
# Markers
# ============================================================================

def pytest_configure(config):
    """Configure custom markers."""
    config.addinivalue_line("markers", "smoke: quick smoke tests")
    config.addinivalue_line("markers", "unit: unit tests")
    config.addinivalue_line("markers", "integration: integration tests")
    config.addinivalue_line("markers", "e2e: end-to-end tests")
    config.addinivalue_line("markers", "fuzz: fuzzing tests")
    config.addinivalue_line("markers", "security: security tests")
    config.addinivalue_line("markers", "benchmark: performance benchmarks")
    config.addinivalue_line("markers", "slow: slow running tests")
    config.addinivalue_line("markers", "external: requires external API")
