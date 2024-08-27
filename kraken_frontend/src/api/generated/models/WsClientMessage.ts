/* tslint:disable */
/* eslint-disable */


import {
    WsClientMessageOneOf1,
    WsClientMessageOneOf1TypeEnum,
    WsClientMessageOneOf1FromJSONTyped,
    WsClientMessageOneOf1ToJSON,
} from './WsClientMessageOneOf1';
import {
    WsClientMessageOneOf,
    WsClientMessageOneOfTypeEnum,
    WsClientMessageOneOfFromJSONTyped,
    WsClientMessageOneOfToJSON,
} from './WsClientMessageOneOf';

/**
 * @type WsClientMessage
 * Message that is sent via websocket by the client
 * @export
 */
export type WsClientMessage = 
  | WsClientMessageOneOf1
  | WsClientMessageOneOf;

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}

const WsClientMessageOneOf1Type = enumToString(WsClientMessageOneOf1TypeEnum);
const WsClientMessageOneOfType = enumToString(WsClientMessageOneOfTypeEnum);

export function WsClientMessageFromJSON(json: any): WsClientMessage {
    return WsClientMessageFromJSONTyped(json, false);
}

export function WsClientMessageFromJSONTyped(json: any, ignoreDiscriminator: boolean): WsClientMessage {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        
        case WsClientMessageOneOf1Type:
            return WsClientMessageOneOf1FromJSONTyped(json, ignoreDiscriminator);
        case WsClientMessageOneOfType:
            return WsClientMessageOneOfFromJSONTyped(json, ignoreDiscriminator);
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
        
        case WsClientMessageOneOf1Type:
            return WsClientMessageOneOf1ToJSON(value);
        case WsClientMessageOneOfType:
            return WsClientMessageOneOfToJSON(value);
        default:
            throw new Error("No variant of WsClientMessage exists with 'type=" + value["type"] + "'");
    }

}
