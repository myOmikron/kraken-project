#!/usr/bin/env python3

import argparse
import json
import os
import shlex
import signal
import subprocess
import sys

DOCKER_COMMANDS = ["up", "down", "pull", "push", "build", "logs", "run", "exec", "ls"]


def signal_handler(_sig, _frame):
    sys.exit(1)


def check_return_code(code):
    if code != 0:
        print(f"Command failed with code {code}", file=sys.stderr)
        sys.exit(code)


def get_branch() -> str:
    process = subprocess.run(["git", "rev-parse", "--abbrev-ref", "HEAD"], stdout=subprocess.PIPE)
    check_return_code(process.returncode)
    return process.stdout.decode("utf-8").strip()


def get_tag() -> str | None:
    process = subprocess.run(["git", "tag", "--points-at"], stdout=subprocess.PIPE)
    check_return_code(process.returncode)
    tags = process.stdout.decode("utf-8").strip().split('\n')
    if len(tags) == 0:
        return None
    return tags[-1]


def get_docker_compose_name() -> str:
    command = shlex.split("docker compose -f docker-compose-dev.yml ls --format json")
    process = subprocess.run(
        command,
        stdout=subprocess.PIPE,
    )
    check_return_code(process.returncode)
    return json.loads(process.stdout.decode("utf-8").strip())[0]["Name"]


def get_webserver_service() -> str | None:
    command = shlex.split("docker compose -f docker-compose-dev.yml ps --format json")
    process = subprocess.run(
        command,
        stdout=subprocess.PIPE,
    )
    check_return_code(process.returncode)
    lines = process.stdout.decode("utf-8").strip()

    if len(lines) == 0:
        return None

    for line in lines.split("\n"):
        info = json.loads(line)

        if "webserver" in [x.split("=")[0] for x in info["Labels"].split(",")]:
            return info["Service"]


def docker_compose_dev(command: str, unknown_args):
    process = subprocess.run(["docker", "compose", "-f", "docker-compose-dev.yml", command, *unknown_args])
    check_return_code(process.returncode)


def docker_compose_prod(command: str, unknown_args):
    process = subprocess.run(["docker", "compose", command, *unknown_args])
    check_return_code(process.returncode)


def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest='command')
    subparsers.required = True

    # docker subcommands for dev
    for c in DOCKER_COMMANDS:
        subparsers.add_parser(c)
    # special subcommands
    subparsers.add_parser("db")
    subparsers.add_parser("make-migrations")
    subparsers.add_parser("gen-api")
    prod = subparsers.add_parser("prod")

    prod_subparsers = prod.add_subparsers(dest='prod_command')
    prod_subparsers.required = True

    # docker subcommands for prod
    for c in DOCKER_COMMANDS:
        prod_subparsers.add_parser(c)
    prod_subparsers.add_parser("db")

    args, unknown_args = parser.parse_known_args()

    signal.signal(signal.SIGINT, signal_handler)

    branch = get_branch()
    os.environ["DEV_TAG"] = branch

    if args.command == "prod":
        tag = get_tag()
        if tag is not None:
            os.environ["PROD_TAG"] = tag

        if args.prod_command == "db":
            docker_compose_prod(
                "exec",
                ["-it", "postgres", "su", "-c", "psql -U $POSTGRES_USER $POSTGRES_DB"],
            )
        else:
            docker_compose_prod(args.prod_command, unknown_args)
    elif args.command == "db":
        docker_compose_dev(
            "exec",
            ["-it", "postgres-dev", "su", "-c", "psql -U $POSTGRES_USER $POSTGRES_DB"],
        )
    elif args.command == "make-migrations":
        webserver_name = get_webserver_service()
        if webserver_name is None:
            print("No service is running. Please run `up` first")
            exit(1)
        docker_compose_dev("exec", ["-it", webserver_name, "server", "make-migrations", *unknown_args])
    elif args.command == "gen-api":
        docker_compose_dev("exec", ["-it", "frontend-dev", "npm", "run", "gen-api"])
    else:
        docker_compose_dev(args.command, unknown_args)


if __name__ == '__main__':
    main()
