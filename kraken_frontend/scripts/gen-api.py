#!/usr/bin/env python3

import os
import pathlib


def main():
    script_file = pathlib.Path(__file__)
    project_root = script_file.parent.parent
    spec = project_root / "openapi.json"
    generated = project_root / "src" / "api" / "generated"
    config = generated / "config.json"

    command = f"npx @openapitools/openapi-generator-cli generate -g typescript-fetch -i {spec} -o {generated} -c {config}"
    print(command)
    os.system(command)

    print("Patch PortOrRange.ts")

    with open(generated / "models" / "PortOrRange.ts", "w") as f:
        f.write("""
export type PortOrRange = number | string;

export function PortOrRangeFromJSON(json: any): PortOrRange {
    return PortOrRangeFromJSONTyped(json, false);
}

export function PortOrRangeFromJSONTyped(json: any, ignoreDiscriminator: boolean): PortOrRange {
    if (json === undefined || json === null || typeof json == "string" || typeof json == "number") {
        return json;
    } else {
        throw TypeError("Invalid json");
    }
}

export function PortOrRangeToJSON(value?: PortOrRange | null): any {
    return value;
}
""")


if __name__ == '__main__':
    main()
