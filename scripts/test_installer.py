"""Offline installer contract tests; no network or user-profile writes."""
import hashlib
import io
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "install.sh"

class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.env = dict(os.environ, HOME=str(self.root), TMPDIR=str(self.root), PATH=str(self.bin), MOCK_ROOT=str(self.root))
        for tool in ["cat", "tr", "bash", "tar", "gzip", "install", "mkdir", "mktemp", "rm", "mv", "awk", "sed", "head", "shasum", "perl", "dirname", "basename"]:
            found = shutil.which(tool)
            if found: (self.bin / tool).symlink_to(found)
        self.mock("uname", '#!/bin/bash\nif [[ "$1" == -s ]]; then echo "${MOCK_OS:-Linux}"; else echo x86_64; fi\n')
        self.mock("curl", '''#!/bin/bash
out=""
while [[ $# -gt 0 ]]; do
  if [[ "$1" == -o ]]; then out="$2"; shift 2; else url="$1"; shift; fi
done
if [[ "$url" == *releases/latest ]]; then echo '{"tag_name":"v3.1.0"}'; exit; fi
if [[ "$url" == *.sha256 ]]; then
  [[ "${MOCK_MISSING:-0}" == 0 ]] || exit 22
  /bin/cp "$MOCK_ROOT/checksum" "$out"
else /bin/cp "$MOCK_ROOT/archive" "$out"; fi
''')
        payload = b'#!/bin/bash\necho "twig 3.1.0"\n'
        with tarfile.open(self.root / "archive", "w:gz") as archive:
            info = tarfile.TarInfo("twig"); info.mode = 0o755; info.size = len(payload)
            archive.addfile(info, io.BytesIO(payload))
        (self.root / "checksum").write_text(hashlib.sha256((self.root / "archive").read_bytes()).hexdigest() + "  archive\n")
    def mock(self, name, body):
        p=self.bin/name; p.write_text(body); p.chmod(0o755)
    def run_installer(self, *args, pipe=False):
        if pipe: return subprocess.run([str(self.bin / "bash"), "-s", "--", *args], input=SCRIPT.read_text(), text=True, capture_output=True, env=self.env)
        return subprocess.run([str(self.bin / "bash"), str(SCRIPT), *args], text=True, capture_output=True, env=self.env)
    def test_help_from_pipe(self):
        p=self.run_installer("--help",pipe=True);self.assertEqual(p.returncode,0,p.stderr);self.assertIn("Usage",p.stdout)
    def test_missing_option_and_unknown(self):
        for flag in ["--version", "--to", "--method", "--unknown"]: self.assertNotEqual(self.run_installer(flag).returncode,0)
    def test_fetch_shasum_fallback_and_cleanup(self):
        self.env["MOCK_OS"]="Darwin"
        dest=self.root/"chosen"
        p=self.run_installer("--to",str(dest),"--yes");self.assertEqual(p.returncode,0,p.stderr);self.assertTrue((dest/"twig").exists());self.assertEqual(list(self.root.glob("twig-install.*")),[])
    def test_wrong_version_preserves_existing_install(self):
        dest=self.root/'chosen';dest.mkdir();(dest/'twig').write_text('previous installation')
        result=self.run_installer('--version','v9.9.9','--to',str(dest),'--yes')
        self.assertNotEqual(result.returncode,0)
        self.assertIn('version mismatch',result.stderr)
        self.assertEqual((dest/'twig').read_text(),'previous installation')
        self.assertEqual(list(self.root.glob('twig-install.*')),[])
    def test_git_bash_points_to_powershell(self):
        self.env['MOCK_OS']='MINGW64_NT-10.0'
        result=self.run_installer('--yes')
        self.assertNotEqual(result.returncode,0);self.assertIn('PowerShell',result.stderr)
    def test_uppercase_checksum(self):
        checksum=self.root/'checksum';checksum.write_text(checksum.read_text().upper())
        result=self.run_installer('--yes');self.assertEqual(result.returncode,0,result.stderr)
    def test_missing_checksum_fails_closed(self):
        self.env["MOCK_MISSING"]="1";self.assertNotEqual(self.run_installer("--yes").returncode,0);self.assertFalse((self.root/".local/bin/twig").exists())
    def test_bad_checksum_fails_closed(self):
        (self.root/"checksum").write_text("0"*64+"  archive\n");self.assertNotEqual(self.run_installer("--yes").returncode,0)
    def test_unsupported_os(self):
        self.env["MOCK_OS"]="FreeBSD";self.assertNotEqual(self.run_installer("--yes").returncode,0)
    def test_build_honors_destination_and_release_tag(self):
        self.mock("cargo", '''#!/bin/bash
printf '%s\\n' "$@" > "$MOCK_ROOT/cargo-args"
while [[ $# -gt 0 ]]; do if [[ "$1" == --root ]]; then root="$2"; break; fi; shift; done
mkdir -p "$root/bin"
printf '#!/bin/bash\\necho twig 3.1.0\\n' > "$root/bin/twig"
/bin/chmod +x "$root/bin/twig"
''')
        dest=self.root/"chosen";p=self.run_installer("--method","build","--to",str(dest),"--yes")
        self.assertEqual(p.returncode,0,p.stderr);self.assertTrue((dest/"twig").exists());self.assertIn("v3.1.0",(self.root/"cargo-args").read_text())

if __name__ == "__main__": unittest.main()
