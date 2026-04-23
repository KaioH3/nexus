"""
Nexus Protocol Python SDK - Example Usage

Run with: python examples/usage.py
Requires: pip install websockets

This demonstrates how easy it's to integrate Nexus Protocol
from any programming language using the Python SDK.
"""

import asyncio
from nexus_protocol import (
    NexusClient,
    SyncNexusClient,
    SandboxPolicy,
    Language,
)


async def example_async():
    """Async usage - the recommended way."""
    print("=== Nexus Protocol Python SDK ===\n")

    # Connect to local Nexus server (or Ollama)
    client = NexusClient("ws://localhost:8080")
    await client.connect()
    print(f"Connected! Session: {client.session_id}")

    # Example 1: Execute Python code
    print("\n[1] Executing Python code...")
    result = await client.execute(
        "print('Hello from Nexus!')\nprint('Python rules!')",
        language="python"
    )
    print(f"stdout: {result.stdout.strip()}")
    print(f"exit_code: {result.exit_code}")
    print(f"execution_time: {result.execution_time_ms}ms")

    # Example 2: Execute with different sandbox policy
    print("\n[2] Executing with zero-trust policy...")
    result = await client.execute(
        "x = 1 + 1",
        language="python",
        policy=SandboxPolicy.zero_trust()
    )
    print(f"stdout: {result.stdout.strip()}")

    # Example 3: Generate with Ollama
    print("\n[3] Generating with Ollama...")
    response = await client.generate(
        "What is the meaning of life? Answer in one sentence.",
        model="llama3.2"
    )
    print(f"response: {response}")

    # Example 4: Streaming generation
    print("\n[4] Streaming generation...")
    stream = await client.generate(
        "Count from 1 to 5:",
        model="llama3.2",
        stream=True
    )
    tokens = []
    async for token in stream:
        tokens.append(token)
        print(token, end="", flush=True)
    print("\n")

    # Example 5: Go code execution
    print("[5] Executing Go code...")
    result = await client.execute(
        '''
package main
import "fmt"
func main() {
    fmt.Println("Hello from Go!")
}
''',
        language="go"
    )
    print(f"stdout: {result.stdout.strip()}")

    await client.close()
    print("\n=== Done! ===")


def example_sync():
    """Sync wrapper for non-async code."""
    print("\n=== Sync Example ===\n")

    with SyncNexusClient("ws://localhost:8080") as client:
        result = client.execute(
            "print('Hello from sync!')",
            language="python"
        )
        print(f"stdout: {result.stdout.strip()}")


if __name__ == "__main__":
    print("Starting examples...\n")
    
    # Async examples
    asyncio.run(example_async())
    
    # Sync examples  
    example_sync()
