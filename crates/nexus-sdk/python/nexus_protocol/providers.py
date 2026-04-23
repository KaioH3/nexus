"""
Nexus AI SDK — Groq Provider Integration

High-performance AI agent with Groq cloud inference.
Includes local Ollama fallback for privacy-first usage.

Example:
    >>> from nexus.providers import GroqProvider
    >>> provider = GroqProvider(api_key="gsk_...")
    >>> result = await provider.generate("Say hello")
"""

from __future__ import annotations

import asyncio
import json
from dataclasses import dataclass, field
from typing import Optional, Iterator, AsyncIterator
from enum import Enum
import aiohttp

__all__ = [
    "GroqProvider",
    "OllamaProvider",
    "OpenAIProvider",
    "Provider",
    "GenerationOptions",
    "GenerationResult",
]


# ============================================================================
# Types
# ============================================================================

@dataclass
class GenerationOptions:
    """Options for text generation."""
    temperature: Optional[float] = 0.7
    max_tokens: Optional[int] = 1024
    top_p: Optional[float] = 0.9
    top_k: Optional[int] = 40
    stop: Optional[list[str]] = None


@dataclass
class GenerationResult:
    """Result from text generation."""
    output: str
    model: str
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int
    duration_ms: int
    cache_hit: bool = False

    def __str__(self) -> str:
        return self.output


# ============================================================================
# Base Provider Interface
# ============================================================================

class Provider:
    """Base interface for LLM providers."""

    async def generate(
        self,
        prompt: str,
        model: str,
        options: Optional[GenerationOptions] = None,
    ) -> GenerationResult:
        raise NotImplementedError

    async def generate_streaming(
        self,
        prompt: str,
        model: str,
        options: Optional[GenerationOptions] = None,
    ) -> AsyncIterator[str]:
        raise NotImplementedError

    async def list_models(self) -> list[dict]:
        raise NotImplementedError


# ============================================================================
# Groq Provider (Cloud - Fastest)
# ============================================================================

class GroqProvider(Provider):
    """
    Groq cloud provider — ultra-fast inference with Llama models.

    Features:
    - Sub-second latency
    - llama-3.3-70b-versatile model
    - Streaming support
    - Cost-effective

    Example:
        >>> provider = GroqProvider(api_key="gsk_...")
        >>> result = await provider.generate("Explain async/await in Python")
        >>> print(result.output)
    """

    BASE_URL = "https://api.groq.com/openai/v1"
    DEFAULT_MODEL = "llama-3.3-70b-versatile"

    def __init__(
        self,
        api_key: str,
        *,
        base_url: Optional[str] = None,
        timeout: int = 60,
    ):
        if not api_key:
            raise ValueError("API key is required for GroqProvider")

        self.api_key = api_key
        self.base_url = base_url or self.BASE_URL
        self.timeout = timeout
        self._client: Optional[aiohttp.ClientSession] = None

    async def _get_client(self) -> aiohttp.ClientSession:
        if self._client is None or self._client.closed:
            self._client = aiohttp.ClientSession(
                timeout=aiohttp.ClientTimeout(total=self.timeout)
            )
        return self._client

    async def generate(
        self,
        prompt: str,
        model: Optional[str] = None,
        options: Optional[GenerationOptions] = None,
    ) -> GenerationResult:
        """
        Generate text with Groq's ultra-fast inference.

        Args:
            prompt: The prompt to generate from
            model: Model name (default: llama-3.3-70b-versatile)
            options: Generation options

        Returns:
            GenerationResult with output and metadata
        """
        import time
        start = time.monotonic()

        model = model or self.DEFAULT_MODEL
        options = options or GenerationOptions()

        body = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": options.temperature,
            "max_tokens": options.max_tokens,
            "top_p": options.top_p,
            "top_k": options.top_k,
            "stream": False,
        }

        if options.stop:
            body["stop"] = options.stop

        client = await self._get_client()

        async with client.post(
            f"{self.base_url}/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json=body,
        ) as resp:
            if resp.status == 401:
                raise ValueError("Invalid Groq API key")
            if resp.status == 429:
                raise ValueError("Rate limit exceeded")
            if resp.status != 200:
                text = await resp.text()
                raise ValueError(f"Groq API error {resp.status}: {text}")

            data = await resp.json()

            duration_ms = int((time.monotonic() - start) * 1000)

            return GenerationResult(
                output=data["choices"][0]["message"]["content"],
                model=data["model"],
                prompt_tokens=data["usage"]["prompt_tokens"],
                completion_tokens=data["usage"]["completion_tokens"],
                total_tokens=data["usage"]["total_tokens"],
                duration_ms=duration_ms,
            )

    async def generate_streaming(
        self,
        prompt: str,
        model: Optional[str] = None,
        options: Optional[GenerationOptions] = None,
    ) -> AsyncIterator[str]:
        """
        Streaming generation with Groq.

        Yields tokens as they are generated.
        """
        model = model or self.DEFAULT_MODEL
        options = options or GenerationOptions()

        body = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": options.temperature,
            "max_tokens": options.max_tokens,
            "top_p": options.top_p,
            "top_k": options.top_k,
            "stream": True,
        }

        client = await self._get_client()

        async with client.post(
            f"{self.base_url}/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json=body,
        ) as resp:
            if resp.status != 200:
                text = await resp.text()
                raise ValueError(f"Groq API error {resp.status}: {text}")

            async for line in resp.content:
                if line:
                    line = line.decode("utf-8").strip()
                    if line.startswith("data:"):
                        if line.startswith("data: [DONE]"):
                            break
                        try:
                            data = json.loads(line[5:])
                            if "choices" in data and len(data["choices"]) > 0:
                                delta = data["choices"][0].get("delta", {})
                                if "content" in delta:
                                    yield delta["content"]
                        except json.JSONDecodeError:
                            continue

    async def list_models(self) -> list[dict]:
        """List available Groq models."""
        client = await self._get_client()

        async with client.get(
            f"{self.base_url}/models",
            headers={"Authorization": f"Bearer {self.api_key}"},
        ) as resp:
            if resp.status != 200:
                return []
            data = await resp.json()
            return data.get("data", [])

    async def close(self) -> None:
        """Close the HTTP client."""
        if self._client and not self._client.closed:
            await self._client.close()


# ============================================================================
# OpenAI Provider (Cloud - GPT Models)
# ============================================================================

class OpenAIProvider(Provider):
    """
    OpenAI cloud provider — GPT-4, GPT-4o, GPT-4o-mini, o1, o1-mini.

    Features:
    - GPT-4o and o1 models
    - Vision support
    - Streaming support
    - Function calling

    Example:
        >>> provider = OpenAIProvider(api_key="sk-...")
        >>> result = await provider.generate("Explain quantum computing")
        >>> print(result.output)
    """

    BASE_URL = "https://api.openai.com/v1"
    DEFAULT_MODEL = "gpt-4o"

    def __init__(
        self,
        api_key: str,
        *,
        base_url: Optional[str] = None,
        timeout: int = 120,
    ):
        if not api_key:
            raise ValueError("API key is required for OpenAIProvider")

        self.api_key = api_key
        self.base_url = base_url or self.BASE_URL
        self.timeout = timeout
        self._client: Optional[aiohttp.ClientSession] = None

    async def _get_client(self) -> aiohttp.ClientSession:
        if self._client is None or self._client.closed:
            self._client = aiohttp.ClientSession(
                timeout=aiohttp.ClientTimeout(total=self.timeout)
            )
        return self._client

    async def generate(
        self,
        prompt: str,
        model: Optional[str] = None,
        options: Optional[GenerationOptions] = None,
    ) -> GenerationResult:
        """
        Generate text with OpenAI's GPT models.

        Args:
            prompt: The prompt to generate from
            model: Model name (default: gpt-4o)
            options: Generation options

        Returns:
            GenerationResult with output and metadata
        """
        import time
        start = time.monotonic()

        model = model or self.DEFAULT_MODEL
        options = options or GenerationOptions()

        body = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": options.temperature,
            "max_tokens": options.max_tokens,
            "top_p": options.top_p,
            "stream": False,
        }

        if options.stop:
            body["stop"] = options.stop

        client = await self._get_client()

        async with client.post(
            f"{self.base_url}/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json=body,
        ) as resp:
            if resp.status == 401:
                raise ValueError("Invalid OpenAI API key")
            if resp.status == 429:
                raise ValueError("OpenAI rate limit exceeded")
            if resp.status != 200:
                text = await resp.text()
                raise ValueError(f"OpenAI API error {resp.status}: {text}")

            data = await resp.json()

            duration_ms = int((time.monotonic() - start) * 1000)

            return GenerationResult(
                output=data["choices"][0]["message"]["content"],
                model=data["model"],
                prompt_tokens=data["usage"]["prompt_tokens"],
                completion_tokens=data["usage"]["completion_tokens"],
                total_tokens=data["usage"]["total_tokens"],
                duration_ms=duration_ms,
            )

    async def generate_streaming(
        self,
        prompt: str,
        model: Optional[str] = None,
        options: Optional[GenerationOptions] = None,
    ) -> AsyncIterator[str]:
        """
        Streaming generation with OpenAI.

        Yields tokens as they are generated.
        """
        model = model or self.DEFAULT_MODEL
        options = options or GenerationOptions()

        body = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": options.temperature,
            "max_tokens": options.max_tokens,
            "top_p": options.top_p,
            "stream": True,
        }

        if options.stop:
            body["stop"] = options.stop

        client = await self._get_client()

        async with client.post(
            f"{self.base_url}/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json=body,
        ) as resp:
            if resp.status != 200:
                text = await resp.text()
                raise ValueError(f"OpenAI API error {resp.status}: {text}")

            async for line in resp.content:
                if line:
                    line = line.decode("utf-8").strip()
                    if line.startswith("data:"):
                        if line.startswith("data: [DONE]"):
                            break
                        try:
                            data = json.loads(line[5:])
                            if "choices" in data and len(data["choices"]) > 0:
                                delta = data["choices"][0].get("delta", {})
                                if "content" in delta:
                                    yield delta["content"]
                        except json.JSONDecodeError:
                            continue

    async def list_models(self) -> list[dict]:
        """List available OpenAI models."""
        client = await self._get_client()

        async with client.get(
            f"{self.base_url}/models",
            headers={"Authorization": f"Bearer {self.api_key}"},
        ) as resp:
            if resp.status != 200:
                return []
            data = await resp.json()
            return data.get("data", [])

    async def close(self) -> None:
        """Close the HTTP client."""
        if self._client and not self._client.closed:
            await self._client.close()


# ============================================================================
# Ollama Provider (Local - Privacy First)
# ============================================================================

class OllamaProvider(Provider):
    """
    Ollama local provider — privacy-first, no data leaves your machine.

    Features:
    - Zero API costs
    - Full privacy (data never leaves)
    - Works offline
    - Many models available

    Example:
        >>> provider = OllamaProvider(base_url="http://localhost:11434")
        >>> result = await provider.generate("Say hello", model="qwen2.5-coder:3b")
    """

    DEFAULT_MODEL = "qwen2.5-coder:3b"

    def __init__(
        self,
        base_url: str = "http://localhost:11434",
        *,
        timeout: int = 120,
    ):
        self.base_url = base_url
        self.timeout = timeout
        self._client: Optional[aiohttp.ClientSession] = None

    async def _get_client(self) -> aiohttp.ClientSession:
        if self._client is None or self._client.closed:
            self._client = aiohttp.ClientSession(
                timeout=aiohttp.ClientTimeout(total=self.timeout)
            )
        return self._client

    async def generate(
        self,
        prompt: str,
        model: Optional[str] = None,
        options: Optional[GenerationOptions] = None,
    ) -> GenerationResult:
        """
        Generate text with local Ollama.

        Args:
            prompt: The prompt to generate from
            model: Model name (default: qwen2.5-coder:3b)
            options: Generation options

        Returns:
            GenerationResult with output and metadata
        """
        import time
        start = time.monotonic()

        model = model or self.DEFAULT_MODEL
        options = options or GenerationOptions()

        body = {
            "model": model,
            "prompt": prompt,
            "stream": False,
        }

        if options.temperature is not None:
            body.setdefault("options", {})["temperature"] = options.temperature
        if options.max_tokens is not None:
            body.setdefault("options", {})["num_predict"] = options.max_tokens
        if options.top_p is not None:
            body.setdefault("options", {})["top_p"] = options.top_p
        if options.top_k is not None:
            body.setdefault("options", {})["top_k"] = options.top_k

        client = await self._get_client()

        async with client.post(
            f"{self.base_url}/api/generate",
            json=body,
        ) as resp:
            if resp.status != 200:
                text = await resp.text()
                raise ValueError(f"Ollama API error {resp.status}: {text}")

            data = await resp.json()

            duration_ms = int((time.monotonic() - start) * 1000)

            return GenerationResult(
                output=data.get("response", ""),
                model=data.get("model", model),
                prompt_tokens=data.get("prompt_eval_count", 0),
                completion_tokens=data.get("eval_count", 0),
                total_tokens=data.get("prompt_eval_count", 0) + data.get("eval_count", 0),
                duration_ms=duration_ms,
            )

    async def generate_streaming(
        self,
        prompt: str,
        model: Optional[str] = None,
        options: Optional[GenerationOptions] = None,
    ) -> AsyncIterator[str]:
        """Streaming generation with Ollama."""
        model = model or self.DEFAULT_MODEL
        options = options or GenerationOptions()

        body = {
            "model": model,
            "prompt": prompt,
            "stream": True,
        }

        client = await self._get_client()

        async with client.post(
            f"{self.base_url}/api/generate",
            json=body,
        ) as resp:
            if resp.status != 200:
                raise ValueError(f"Ollama API error {resp.status}")

            async for line in resp.content:
                if line:
                    try:
                        data = json.loads(line)
                        if "response" in data:
                            yield data["response"]
                        if data.get("done"):
                            break
                    except json.JSONDecodeError:
                        continue

    async def list_models(self) -> list[dict]:
        """List available Ollama models."""
        client = await self._get_client()

        async with client.get(f"{self.base_url}/api/tags") as resp:
            if resp.status != 200:
                return []
            data = await resp.json()
            return data.get("models", [])

    async def close(self) -> None:
        """Close the HTTP client."""
        if self._client and not self._client.closed:
            await self._client.close()
