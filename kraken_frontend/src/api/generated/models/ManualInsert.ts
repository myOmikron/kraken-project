/* tslint:disable */
/* eslint-disable */


import {
    ManualInsertOneOf4,
    ManualInsertOneOf4TypeEnum,
    ManualInsertOneOf4FromJSONTyped,
    ManualInsertOneOf4ToJSON,
} from './ManualInsertOneOf4';
import {
    ManualInsertOneOf2,
    ManualInsertOneOf2TypeEnum,
    ManualInsertOneOf2FromJSONTyped,
    ManualInsertOneOf2ToJSON,
} from './ManualInsertOneOf2';
import {
    ManualInsertOneOf1,
    ManualInsertOneOf1TypeEnum,
    ManualInsertOneOf1FromJSONTyped,
    ManualInsertOneOf1ToJSON,
} from './ManualInsertOneOf1';
import {
    ManualInsertOneOf3,
    ManualInsertOneOf3TypeEnum,
    ManualInsertOneOf3FromJSONTyped,
    ManualInsertOneOf3ToJSON,
} from './ManualInsertOneOf3';
import {
    ManualInsertOneOf,
    ManualInsertOneOfTypeEnum,
    ManualInsertOneOfFromJSONTyped,
    ManualInsertOneOfToJSON,
} from './ManualInsertOneOf';

/**
 * @type ManualInsert
 * Message that is sent via websocket
 * @export
 */
export type ManualInsert = 
  | ManualInsertOneOf4
  | ManualInsertOneOf2
  | ManualInsertOneOf1
  | ManualInsertOneOf3
  | ManualInsertOneOf;

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}

const ManualInsertOneOf4Type = enumToString(ManualInsertOneOf4TypeEnum);
const ManualInsertOneOf2Type = enumToString(ManualInsertOneOf2TypeEnum);
const ManualInsertOneOf1Type = enumToString(ManualInsertOneOf1TypeEnum);
const ManualInsertOneOf3Type = enumToString(ManualInsertOneOf3TypeEnum);
const ManualInsertOneOfType = enumToString(ManualInsertOneOfTypeEnum);

export function ManualInsertFromJSON(json: any): ManualInsert {
    return ManualInsertFromJSONTyped(json, false);
}

export function ManualInsertFromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualInsert {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['type']) {
        
        case ManualInsertOneOf4Type:
            return ManualInsertOneOf4FromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOf2Type:
            return ManualInsertOneOf2FromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOf1Type:
            return ManualInsertOneOf1FromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOf3Type:
            return ManualInsertOneOf3FromJSONTyped(json, ignoreDiscriminator);
        case ManualInsertOneOfType:
            return ManualInsertOneOfFromJSONTyped(json, ignoreDiscriminator);
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
        
        case ManualInsertOneOf4Type:
            return ManualInsertOneOf4ToJSON(value);
        case ManualInsertOneOf2Type:
            return ManualInsertOneOf2ToJSON(value);
        case ManualInsertOneOf1Type:
            return ManualInsertOneOf1ToJSON(value);
        case ManualInsertOneOf3Type:
            return ManualInsertOneOf3ToJSON(value);
        case ManualInsertOneOfType:
            return ManualInsertOneOfToJSON(value);
        default:
            throw new Error("No variant of ManualInsert exists with 'type=" + value["type"] + "'");
    }

}
