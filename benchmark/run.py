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


all_commands = [
    (
        "biome",
        Path("./biome-test"),
        [
            "npx biome format --write ./src/{size}.tsx",
            "./node_modules/@biomejs/cli-linux-x64-musl/biome format --write ./src/{size}.tsx",
            "{foro} format ./src/{size}.tsx",
        ],
    ),
    (
        "ruff format",
        Path("./ruff-test"),
        [
            "ruff format ./src/ruff_test/{size}.py",
            "{foro} format ./src/ruff_test/{size}.py",
        ],
    ),
    (
        "clang-format",
        Path("./clang-format-test"),
        [
            "clang-format ./{size}.cpp",
            "{foro} format ./{size}.cpp",
        ],
    ),
]


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="foro-benchmark-run",
        description="Run a benchmark comparing the forum with other methods in a real project.",
    )

    parser.add_argument(
        "foro-command",
        nargs="?",
        default="foro",
        help="The command to run foro.",
    )

    args = parser.parse_args()

    foro = getattr(args, "foro-command")

    if not exists_hyperfine():
        print(
            "hyperfine is not installed. Please install it before running this script.",
            file=sys.stderr,
        )
        sys.exit(1)
        return

    print("Running benchmarks...", flush=True)

    for name, project, commands in all_commands:
        for size in ("small", "large"):
            print(f"\n\n\nRunning benchmark for {name} + {size}...\n", flush=True)

            p_command = map(lambda x: x.format(size=size, foro=foro), commands)

            hyperfine_command = [
                "hyperfine",
                "-N",
                "--time-unit",
                "microsecond",
                "--style",
                "basic",
                "--warmup",
                "1",
                *p_command,
            ]

            res = subprocess.run(hyperfine_command, cwd=project)


if __name__ == "__main__":
    main()
