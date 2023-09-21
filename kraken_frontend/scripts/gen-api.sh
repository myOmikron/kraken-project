#!/usr/bin/env bash

set -e

PROJECT_ROOT=$(dirname "$(dirname \""$0"\")")
SPEC="${PROJECT_ROOT}/openapi.json"
GENERATED="${PROJECT_ROOT}/src/api/generated"
CONFIG="${GENERATED}/config.json"
TMP="${PROJECT_ROOT}/tmp"

mkdir -p "${TMP}"
mv "${GENERATED}/config.json" "${GENERATED}/.openapi-generator-ignore" "${GENERATED}/.openapi-generator" "${GENERATED}/README.md" "${TMP}"
rm -rf "$GENERATED"
mkdir -p "$GENERATED"
mv "${TMP}/config.json" "${TMP}/.openapi-generator-ignore" "${TMP}/.openapi-generator" "${TMP}/README.md" "${GENERATED}"

npx @openapitools/openapi-generator-cli generate -g typescript-fetch -i "${SPEC}" -o "${GENERATED}" -c "${CONFIG}"

echo "Patch PortOrRange.ts"

cat > "${GENERATED}/models/PortOrRange.ts" << EOF
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
EOF
