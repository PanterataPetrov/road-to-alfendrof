#!/usr/bin/env python3
"""Emit Actions version outputs; never reuse a stable tag for another main commit."""

import os
import re
import subprocess
import tomllib
from pathlib import Path


with Path("Cargo.toml").open("rb") as manifest:
    version = tomllib.load(manifest)["package"]["version"]
if not re.fullmatch(r"\d+\.\d+\.\d+", version):
    raise SystemExit("Stable releases require a MAJOR.MINOR.PATCH version in Cargo.toml.")
tag = f"v{version}"

# All tags must be fetched by checkout. Artifact-only builds on other refs do not reserve a version.
if os.environ.get("GITHUB_REF") == "refs/heads/main":
    existing = subprocess.run(["git", "show-ref", "--verify", "--quiet", f"refs/tags/{tag}"])
    if existing.returncode == 0:
        tagged_commit = subprocess.check_output(
            ["git", "rev-parse", f"refs/tags/{tag}^{{commit}}"], text=True
        ).strip()
        if tagged_commit != os.environ["GITHUB_SHA"]:
            raise SystemExit(
                f"{tag} already identifies another commit. Bump the version in Cargo.toml, "
                "update Cargo.lock and the manual Inno Setup default, then merge to main. "
                "Existing stable tags are never moved."
            )
    elif existing.returncode != 1:
        existing.check_returncode()

print(f"version={version}")
print(f"tag={tag}")
