/* tslint:disable */
/* eslint-disable */


import {
    WsMessageOneOf,
    WsMessageOneOfTypeEnum,
    WsMessageOneOfFromJSONTyped,
    WsMessageOneOfToJSON,
} from './WsMessageOneOf';
import {
    WsMessageOneOf1,
    WsMessageOneOf1TypeEnum,
    WsMessageOneOf1FromJSONTyped,
    WsMessageOneOf1ToJSON,
} from './WsMessageOneOf1';
import {
    WsMessageOneOf10,
    WsMessageOneOf10TypeEnum,
    WsMessageOneOf10FromJSONTyped,
    WsMessageOneOf10ToJSON,
} from './WsMessageOneOf10';
import {
    WsMessageOneOf11,
    WsMessageOneOf11TypeEnum,
    WsMessageOneOf11FromJSONTyped,
    WsMessageOneOf11ToJSON,
} from './WsMessageOneOf11';
import {
    WsMessageOneOf2,
    WsMessageOneOf2TypeEnum,
    WsMessageOneOf2FromJSONTyped,
    WsMessageOneOf2ToJSON,
} from './WsMessageOneOf2';
import {
    WsMessageOneOf3,
    WsMessageOneOf3TypeEnum,
    WsMessageOneOf3FromJSONTyped,
    WsMessageOneOf3ToJSON,
} from './WsMessageOneOf3';
import {
    WsMessageOneOf4,
    WsMessageOneOf4TypeEnum,
    WsMessageOneOf4FromJSONTyped,
    WsMessageOneOf4ToJSON,
} from './WsMessageOneOf4';
import {
    WsMessageOneOf5,
    WsMessageOneOf5TypeEnum,
    WsMessageOneOf5FromJSONTyped,
    WsMessageOneOf5ToJSON,
} from './WsMessageOneOf5';
import {
    WsMessageOneOf6,
    WsMessageOneOf6TypeEnum,
    WsMessageOneOf6FromJSONTyped,
    WsMessageOneOf6ToJSON,
} from './WsMessageOneOf6';
import {
    WsMessageOneOf7,
    WsMessageOneOf7TypeEnum,
    WsMessageOneOf7FromJSONTyped,
    WsMessageOneOf7ToJSON,
} from './WsMessageOneOf7';
import {
    WsMessageOneOf8,
    WsMessageOneOf8TypeEnum,
    WsMessageOneOf8FromJSONTyped,
    WsMessageOneOf8ToJSON,
} from './WsMessageOneOf8';
import {
    WsMessageOneOf9,
    WsMessageOneOf9TypeEnum,
    WsMessageOneOf9FromJSONTyped,
    WsMessageOneOf9ToJSON,
} from './WsMessageOneOf9';

/**
 * @type WsMessage
 * Message that is sent via websocket
 * @export
 */
export type WsMessage = 
  | WsMessageOneOf
  | WsMessageOneOf1
  | WsMessageOneOf10
  | WsMessageOneOf11
  | WsMessageOneOf2
  | WsMessageOneOf3
  | WsMessageOneOf4
  | WsMessageOneOf5
  | WsMessageOneOf6
  | WsMessageOneOf7
  | WsMessageOneOf8
  | WsMessageOneOf9;

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}

const WsMessageOneOfType = enumToString(WsMessageOneOfTypeEnum);
const WsMessageOneOf1Type = enumToString(WsMessageOneOf1TypeEnum);
const WsMessageOneOf10Type = enumToString(WsMessageOneOf10TypeEnum);
const WsMessageOneOf11Type = enumToString(WsMessageOneOf11TypeEnum);
const WsMessageOneOf2Type = enumToString(WsMessageOneOf2TypeEnum);
const WsMessageOneOf3Type = enumToString(WsMessageOneOf3TypeEnum);
const WsMessageOneOf4Type = enumToString(WsMessageOneOf4TypeEnum);
const WsMessageOneOf5Type = enumToString(WsMessageOneOf5TypeEnum);
const WsMessageOneOf6Type = enumToString(WsMessageOneOf6TypeEnum);
const WsMessageOneOf7Type = enumToString(WsMessageOneOf7TypeEnum);
const WsMessageOneOf8Type = enumToString(WsMessageOneOf8TypeEnum);
const WsMessageOneOf9Type = enumToString(WsMessageOneOf9TypeEnum);

export function WsMessageFromJSON(json: any): WsMessage {
    return WsMessageFromJSONTyped(json, false);
}

export function WsMessageFromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessage {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        
        case WsMessageOneOfType:
            return WsMessageOneOfFromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf1Type:
            return WsMessageOneOf1FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf10Type:
            return WsMessageOneOf10FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf11Type:
            return WsMessageOneOf11FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf2Type:
            return WsMessageOneOf2FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf3Type:
            return WsMessageOneOf3FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf4Type:
            return WsMessageOneOf4FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf5Type:
            return WsMessageOneOf5FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf6Type:
            return WsMessageOneOf6FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf7Type:
            return WsMessageOneOf7FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf8Type:
            return WsMessageOneOf8FromJSONTyped(json, ignoreDiscriminator);
        case WsMessageOneOf9Type:
            return WsMessageOneOf9FromJSONTyped(json, ignoreDiscriminator);
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
        
        case WsMessageOneOfType:
            return WsMessageOneOfToJSON(value);
        case WsMessageOneOf1Type:
            return WsMessageOneOf1ToJSON(value);
        case WsMessageOneOf10Type:
            return WsMessageOneOf10ToJSON(value);
        case WsMessageOneOf11Type:
            return WsMessageOneOf11ToJSON(value);
        case WsMessageOneOf2Type:
            return WsMessageOneOf2ToJSON(value);
        case WsMessageOneOf3Type:
            return WsMessageOneOf3ToJSON(value);
        case WsMessageOneOf4Type:
            return WsMessageOneOf4ToJSON(value);
        case WsMessageOneOf5Type:
            return WsMessageOneOf5ToJSON(value);
        case WsMessageOneOf6Type:
            return WsMessageOneOf6ToJSON(value);
        case WsMessageOneOf7Type:
            return WsMessageOneOf7ToJSON(value);
        case WsMessageOneOf8Type:
            return WsMessageOneOf8ToJSON(value);
        case WsMessageOneOf9Type:
            return WsMessageOneOf9ToJSON(value);
        default:
            throw new Error("No variant of WsMessage exists with 'type=" + value["type"] + "'");
    }

}
