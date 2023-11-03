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

echo "Patch WsMessage.ts"
mapfile -t VARIANTS < <(find "${GENERATED}/models/" | grep -o 'WsMessageOneOf[0-9]*')

IMPORTS=""
TYPE_DECL=""
EXTRACT_ENUMS=""
FROM_JSON_CASES=""
TO_JSON_CASES=""
for V in "${VARIANTS[@]}"; do
IMPORTS+="
import {
    ${V},
    ${V}TypeEnum,
    ${V}FromJSONTyped,
    ${V}ToJSON,
} from './${V}';"
TYPE_DECL+="
  | ${V}"
EXTRACT_ENUMS+="
const ${V}Type = enumToString(${V}TypeEnum);"
FROM_JSON_CASES+="
        case ${V}Type:
            return ${V}FromJSONTyped(json, ignoreDiscriminator);"
TO_JSON_CASES+="
        case ${V}Type:
            return ${V}ToJSON(value);"
done

cat > "${GENERATED}/models/WsMessage.ts" << EOF
/* tslint:disable */
/* eslint-disable */

${IMPORTS}

/**
 * @type WsMessage
 * Message that is sent via websocket
 * @export
 */
export type WsMessage = ${TYPE_DECL};

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}
${EXTRACT_ENUMS}

export function WsMessageFromJSON(json: any): WsMessage {
    return WsMessageFromJSONTyped(json, false);
}

export function WsMessageFromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessage {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        ${FROM_JSON_CASES}
        default:
            throw new Error("No variant of WsMessage exists with 'type=" + json["type"] + "'");
    }
}

export function WsMessageToJSON(value?: WsMessage | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    switch (value['type']) {
        ${TO_JSON_CASES}
        default:
            throw new Error("No variant of WsMessage exists with 'type=" + value["type"] + "'");
    }

}
EOF
