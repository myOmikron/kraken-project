/* tslint:disable */
/* eslint-disable */


import {
    ManualInsertOneOf3,
    ManualInsertOneOf3TypeEnum,
    ManualInsertOneOf3FromJSONTyped,
    ManualInsertOneOf3ToJSON,
} from './ManualInsertOneOf3';
import {
    ManualInsertOneOf1,
    ManualInsertOneOf1TypeEnum,
    ManualInsertOneOf1FromJSONTyped,
    ManualInsertOneOf1ToJSON,
} from './ManualInsertOneOf1';
import {
    ManualInsertOneOf,
    ManualInsertOneOfTypeEnum,
    ManualInsertOneOfFromJSONTyped,
    ManualInsertOneOfToJSON,
} from './ManualInsertOneOf';
import {
    ManualInsertOneOf2,
    ManualInsertOneOf2TypeEnum,
    ManualInsertOneOf2FromJSONTyped,
    ManualInsertOneOf2ToJSON,
} from './ManualInsertOneOf2';

/**
 * @type ManualInsert
 * Message that is sent via websocket
 * @export
 */
export type ManualInsert = 
  | ManualInsertOneOf3
  | ManualInsertOneOf1
  | ManualInsertOneOf
  | ManualInsertOneOf2;

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}

const ManualInsertOneOf3Type = enumToString(ManualInsertOneOf3TypeEnum);
const ManualInsertOneOf1Type = enumToString(ManualInsertOneOf1TypeEnum);
const ManualInsertOneOfType = enumToString(ManualInsertOneOfTypeEnum);
const ManualInsertOneOf2Type = enumToString(ManualInsertOneOf2TypeEnum);

export function ManualInsertFromJSON(json: any): ManualInsert {
    return ManualInsertFromJSONTyped(json, false);
}

export function ManualInsertFromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualInsert {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        
        case ManualInsertOneOf3Type:
            return ManualInsertOneOf3FromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOf1Type:
            return ManualInsertOneOf1FromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOfType:
            return ManualInsertOneOfFromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOf2Type:
            return ManualInsertOneOf2FromJSONTyped(json, ignoreDiscriminator);
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
        
        case ManualInsertOneOf3Type:
            return ManualInsertOneOf3ToJSON(value);
        case ManualInsertOneOf1Type:
            return ManualInsertOneOf1ToJSON(value);
        case ManualInsertOneOfType:
            return ManualInsertOneOfToJSON(value);
        case ManualInsertOneOf2Type:
            return ManualInsertOneOf2ToJSON(value);
        default:
            throw new Error("No variant of ManualInsert exists with 'type=" + value["type"] + "'");
    }

}
