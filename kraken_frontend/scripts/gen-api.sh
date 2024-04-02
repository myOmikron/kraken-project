#!/usr/bin/env bash

set -e

PROJECT_ROOT=$(dirname "$(dirname \""$0"\")")
SPEC="${PROJECT_ROOT}/openapi.json"
GENERATED="${PROJECT_ROOT}/src/api/generated"
CONFIG="${GENERATED}/config.json"
TMP="${PROJECT_ROOT}/tmp"

# Delete everything in GENERATED except for a few config files
mkdir -p "${TMP}"
mv "${GENERATED}/config.json" "${GENERATED}/.openapi-generator-ignore" "${GENERATED}/.openapi-generator" "${GENERATED}/README.md" "${TMP}"
rm -rf "$GENERATED"
mkdir -p "$GENERATED"
mv "${TMP}/config.json" "${TMP}/.openapi-generator-ignore" "${TMP}/.openapi-generator" "${TMP}/README.md" "${GENERATED}"

# Tweak the representation of string enums because our generator doesn't like the new one
jq '.components.schemas |= map_values( if .oneOf != null and all(.oneOf[]; .type == "string") then { description: .description, type: "string", enum: [ .oneOf[].enum[0] ] } else . end )' "${SPEC}" > "${TMP}/openapi.json"
mv "${TMP}/openapi.json" "${SPEC}"
npx prettier --write "${SPEC}"

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

echo "Patch DomainOrNetwork.ts"
cat > "${GENERATED}/models/DomainOrNetwork.ts" << EOF
/**
 * @type DomainOrNetwork
 * Either an ip address / network or a domain name
 * @export
 */
export type DomainOrNetwork = string;

export function DomainOrNetworkFromJSON(json: any): DomainOrNetwork {
    return DomainOrNetworkFromJSONTyped(json, false);
}

export function DomainOrNetworkFromJSONTyped(json: any, ignoreDiscriminator: boolean): DomainOrNetwork {
        if (json === undefined || json === null || typeof json == "string") {
            return json;
        } else {
            throw TypeError("Invalid json");
        }
}

export function DomainOrNetworkToJSON(value?: DomainOrNetwork | null): any {
    return value;
}
EOF