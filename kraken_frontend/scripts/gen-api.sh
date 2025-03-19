#!/usr/bin/env bash

set -e

PROJECT_ROOT=$(dirname "$(dirname \""$0"\")")
SPEC="${PROJECT_ROOT}/openapi.json"
GENERATED="${PROJECT_ROOT}/src/api/generated"
CONFIG="${GENERATED}/config.json"
TMP="${PROJECT_ROOT}/tmp"

wget --no-check-certificate "http://nginx-dev/api-doc/frontend-api.json" -O "$SPEC"

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

echo "Patch WsMessage.ts"
mapfile -t VARIANTS < <(find "${GENERATED}/models/" | grep -o 'WsMessageOneOf[0-9]*' | sort)

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


echo "Patch WsClientMessage.ts"
mapfile -t VARIANTS < <(find "${GENERATED}/models/" | grep -o 'WsClientMessageOneOf[0-9]*' | sort)

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

cat > "${GENERATED}/models/WsClientMessage.ts" << EOF
/* tslint:disable */
/* eslint-disable */

${IMPORTS}

/**
 * @type WsClientMessage
 * Message that is sent via websocket by the client
 * @export
 */
export type WsClientMessage = ${TYPE_DECL};

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}
${EXTRACT_ENUMS}

export function WsClientMessageFromJSON(json: any): WsClientMessage {
    return WsClientMessageFromJSONTyped(json, false);
}

export function WsClientMessageFromJSONTyped(json: any, ignoreDiscriminator: boolean): WsClientMessage {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        ${FROM_JSON_CASES}
        default:
            throw new Error("No variant of WsClientMessage exists with 'type=" + json["type"] + "'");
    }
}

export function WsClientMessageToJSON(value?: WsClientMessage | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    switch (value['type']) {
        ${TO_JSON_CASES}
        default:
            throw new Error("No variant of WsClientMessage exists with 'type=" + value["type"] + "'");
    }

}
EOF

echo "Patch SourceAttackResult.ts"
mapfile -t VARIANTS < <(find "${GENERATED}/models/" | grep -o 'SourceAttackResultOneOf[0-9]*' | sort)

IMPORTS=""
TYPE_DECL=""
EXTRACT_ENUMS=""
FROM_JSON_CASES=""
TO_JSON_CASES=""
for V in "${VARIANTS[@]}"; do
IMPORTS+="
import {
    ${V},
    ${V}AttackTypeEnum,
    ${V}FromJSONTyped,
    ${V}ToJSON,
} from './${V}';"
TYPE_DECL+="
  | ${V}"
EXTRACT_ENUMS+="
const ${V}AttackType = enumToString(${V}AttackTypeEnum);"
FROM_JSON_CASES+="
        case ${V}AttackType:
            return ${V}FromJSONTyped(json, ignoreDiscriminator);"
TO_JSON_CASES+="
        case ${V}AttackType:
            return ${V}ToJSON(value);"
done

cat > "${GENERATED}/models/SourceAttackResult.ts" << EOF
/* tslint:disable */
/* eslint-disable */

${IMPORTS}

/**
 * @type SourceAttackResult
 * @export
 */
export type SourceAttackResult = ${TYPE_DECL};

function enumToString<K extends string, V extends string>(obj: Record<K, V>): V {
    // @ts-ignore
    return Object.values(obj)[0];
}
${EXTRACT_ENUMS}

export function SourceAttackResultFromJSON(json: any): SourceAttackResult {
    return SourceAttackResultFromJSONTyped(json, false);
}

export function SourceAttackResultFromJSONTyped(json: any, ignoreDiscriminator: boolean): SourceAttackResult {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['attack_type']) {
        ${FROM_JSON_CASES}
        default:
            throw new Error("No variant of SourceAttackResult exists with 'attackType=" + json["attack_type"] + "'");
    }
}

export function SourceAttackResultToJSON(value?: SourceAttackResult | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    switch (value['attackType']) {
        ${TO_JSON_CASES}
        default:
            throw new Error("No variant of SourceAttackResult exists with 'attackType=" + value["attackType"] + "'");
    }

}
EOF

echo "Patch SourceAttack.ts"+
sed -i 's/export interface SourceAttack extends SourceAttackResult/export type SourceAttack = SourceAttackResult \&/' "${GENERATED}/models/SourceAttack.ts"

echo "Patch ManualInsert.ts"
mapfile -t VARIANTS < <(find "${GENERATED}/models/" | grep -o 'ManualInsertOneOf[0-9]*' | sort)

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

cat > "${GENERATED}/models/ManualInsert.ts" << EOF
/* tslint:disable */
/* eslint-disable */

${IMPORTS}

/**
 * @type ManualInsert
 * Message that is sent via websocket
 * @export
 */
export type ManualInsert = ${TYPE_DECL};

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}
${EXTRACT_ENUMS}

export function ManualInsertFromJSON(json: any): ManualInsert {
    return ManualInsertFromJSONTyped(json, false);
}

export function ManualInsertFromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualInsert {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        ${FROM_JSON_CASES}
        default:
            throw new Error("No variant of ManualInsert exists with 'type=" + json["type"] + "'");
    }
}

export function ManualInsertToJSON(value?: ManualInsert | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    switch (value['type']) {
        ${TO_JSON_CASES}
        default:
            throw new Error("No variant of ManualInsert exists with 'type=" + value["type"] + "'");
    }

}
EOF

npx prettier --write "${SPEC}"