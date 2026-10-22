#!/usr/bin/env python3
"""Package the allowlisted same-origin UI files for a private GitHub release."""
import argparse, hashlib, pathlib, zipfile
ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "rust-arr" / "src"
FILES = [line.strip() for line in (ROOT / "rust-arr" / "ui-bundle-files.txt").read_text().splitlines() if line.strip() and not line.startswith("#")]
def package(output):
    output = pathlib.Path(output); output.parent.mkdir(parents=True, exist_ok=True)
    missing = [p for p in FILES if not (SOURCE / p).is_file()]
    if missing: raise ValueError("missing UI bundle files: " + ", ".join(missing))
    with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED) as archive:
        for name in FILES: archive.write(SOURCE / name, name)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_suffix(output.suffix + ".sha256").write_text(f"{digest}  {output.name}\n")
    print(f"packaged {len(FILES)} UI files: {output} sha256={digest}")
def main():
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument("--output", required=True); args = parser.parse_args()
    try: package(args.output)
    except (OSError, ValueError) as exc: parser.error(str(exc))
if __name__ == "__main__": main()
