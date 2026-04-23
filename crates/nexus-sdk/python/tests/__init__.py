"""
Nexus Protocol — Python SDK Test Suite

Organized in layers:
1. Smoke tests - quick sanity checks
2. Unit tests - isolated components
3. Integration tests - component interaction
4. E2E tests - full workflow
5. Fuzzing - random input testing
6. Security - attack vectors
7. Benchmarks - performance
"""

import asyncio
import os
import sys
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

pytest_plugins = ["pytest_asyncio"]

__version__ = "0.2.0"
