# usage: uv run ./run.py > ./run_result.txt

import argparse
import subprocess
import sys
from pathlib import Path


def exists_hyperfine() -> bool:
    return (
        subprocess.run(
            "hyperfine --version", shell=True, check=False, capture_output=True
        ).returncode
        == 0
    )


def config_projects(foro: Path) -> list[tuple[Path, list[str]]]:
    return [
        [
            "biome",
            Path("./biome-test"),
            [
                "npx biome format --write ./test.tsx",
                "./node_modules/@biomejs/cli-linux-x64-musl/biome format --write ./test.tsx",
                f"{foro} format ./test.tsx",
            ],
        ],
        [
            "ruff format",
            Path("./ruff-test"),
            [
                "ruff format ./src/ruff_test/main.py",
                f"{foro} format ./src/ruff_test/main.py",
            ],
        ],
    ]


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="foro-benchmark-run",
        description="Run a benchmark comparing the forum with other methods in a real project.",
    )

    parser.add_argument(
        "foro-command",
        nargs="?",
        default="../target/release/foro",
        help="The command to run foro.",
    )

    args = parser.parse_args()

    foro = Path(getattr(args, "foro-command")).resolve()
    projects = config_projects(foro)

    if not exists_hyperfine():
        print(
            "hyperfine is not installed. Please install it before running this script.",
            file=sys.stderr,
        )
        sys.exit(1)
        return

    print("Running benchmarks...", flush=True)

    for name, project, commands in projects:
        print(f"\n\n\nRunning benchmark for {name}...\n", flush=True)

        hyperfine_command = [
            "hyperfine",
            "-N",
            "--time-unit",
            "microsecond",
            "--style",
            "basic",
            *commands,
        ]

        res = subprocess.run(hyperfine_command, cwd=project)


if __name__ == "__main__":
    main()
