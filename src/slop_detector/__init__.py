"""slop-detector: Fast regex-based detection of AI prose tells."""

from .config import Config, load_config
from .detector import analyze
from .models import SlopFlag, SlopResult
from .voice import generate_voice_directive

__all__ = [
    "analyze",
    "Config",
    "generate_voice_directive",
    "load_config",
    "SlopFlag",
    "SlopResult",
]
