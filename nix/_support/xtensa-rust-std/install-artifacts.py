"""target library artifacts from an isolated build-std Cargo build."""

import json
from pathlib import Path
import shutil
import sys


def install_artifacts(messages: Path, target_output: Path, destination: Path) -> None:
    target_output = target_output.resolve()
    artifacts = {}
    crates = set()
    observed = []
    succeeded = False

    for line in messages.read_text().splitlines():
        message = json.loads(line)
        if message.get("reason") == "build-finished":
            succeeded = message["success"]
        if message.get("reason") != "compiler-artifact":
            continue

        crate = message["target"]["name"]
        observed.append((crate, message["filenames"]))
        if crate == "xtensa_sysroot_probe":
            continue

        for filename in message["filenames"]:
            artifact = Path(filename).resolve()
            if not artifact.is_relative_to(target_output) or artifact.suffix not in {
                ".rlib", ".rmeta"
            }:
                continue
            key = (crate, artifact.suffix)
            if key in artifacts and artifacts[key] != artifact:
                raise RuntimeError(f"Multiple library variants for {crate}")
            artifacts[key] = artifact
            if artifact.suffix == ".rlib":
                crates.add(crate)

    if not succeeded:
        raise RuntimeError("Cargos build failed")
    missing = {"core", "alloc", "compiler_builtins"} - crates
    if missing:
        raise RuntimeError(
            f"Missing target libraries: {sorted(missing)}; "
            f"expected archives to be at {target_output}. Cargo artifacts: {observed}"
        )

    destination.mkdir(parents=True, exist_ok=True)
    for (crate, _), artifact in sorted(artifacts.items()):
        shutil.copyfile(artifact, destination / artifact.name)
        print(f"Installed {crate}: {artifact.name}")


if __name__ == "__main__":
    install_artifacts(*(Path(argument) for argument in sys.argv[1:]))
