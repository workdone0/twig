"""Build the portable Rust engine; wasm-bindgen CLI must match Cargo.toml."""
from pathlib import Path
import subprocess
ROOT = Path(__file__).resolve().parents[1]
subprocess.run(['cargo', 'build', '--locked', '-p', 'twig-wasm', '--target', 'wasm32-unknown-unknown', '--release'], cwd=ROOT, check=True)
subprocess.run(['wasm-bindgen', '--target', 'web', '--out-dir', str(ROOT/'web/src/wasm'), str(ROOT/'target/wasm32-unknown-unknown/release/twig_wasm.wasm')], check=True)
