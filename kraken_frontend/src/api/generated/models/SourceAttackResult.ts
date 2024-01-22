/* tslint:disable */
/* eslint-disable */


import {
    SourceAttackResultOneOf,
    SourceAttackResultOneOfAttackTypeEnum,
    SourceAttackResultOneOfFromJSONTyped,
    SourceAttackResultOneOfToJSON,
} from './SourceAttackResultOneOf';
import {
    SourceAttackResultOneOf1,
    SourceAttackResultOneOf1AttackTypeEnum,
    SourceAttackResultOneOf1FromJSONTyped,
    SourceAttackResultOneOf1ToJSON,
} from './SourceAttackResultOneOf1';
import {
    SourceAttackResultOneOf2,
    SourceAttackResultOneOf2AttackTypeEnum,
    SourceAttackResultOneOf2FromJSONTyped,
    SourceAttackResultOneOf2ToJSON,
} from './SourceAttackResultOneOf2';
import {
    SourceAttackResultOneOf3,
    SourceAttackResultOneOf3AttackTypeEnum,
    SourceAttackResultOneOf3FromJSONTyped,
    SourceAttackResultOneOf3ToJSON,
} from './SourceAttackResultOneOf3';
import {
    SourceAttackResultOneOf4,
    SourceAttackResultOneOf4AttackTypeEnum,
    SourceAttackResultOneOf4FromJSONTyped,
    SourceAttackResultOneOf4ToJSON,
} from './SourceAttackResultOneOf4';
import {
    SourceAttackResultOneOf5,
    SourceAttackResultOneOf5AttackTypeEnum,
    SourceAttackResultOneOf5FromJSONTyped,
    SourceAttackResultOneOf5ToJSON,
} from './SourceAttackResultOneOf5';
import {
    SourceAttackResultOneOf6,
    SourceAttackResultOneOf6AttackTypeEnum,
    SourceAttackResultOneOf6FromJSONTyped,
    SourceAttackResultOneOf6ToJSON,
} from './SourceAttackResultOneOf6';
import {
    SourceAttackResultOneOf7,
    SourceAttackResultOneOf7AttackTypeEnum,
    SourceAttackResultOneOf7FromJSONTyped,
    SourceAttackResultOneOf7ToJSON,
} from './SourceAttackResultOneOf7';
import {
    SourceAttackResultOneOf8,
    SourceAttackResultOneOf8AttackTypeEnum,
    SourceAttackResultOneOf8FromJSONTyped,
    SourceAttackResultOneOf8ToJSON,
} from './SourceAttackResultOneOf8';

/**
 * @type SourceAttackResult
 * @export
 */
export type SourceAttackResult = 
  | SourceAttackResultOneOf
  | SourceAttackResultOneOf1
  | SourceAttackResultOneOf2
  | SourceAttackResultOneOf3
  | SourceAttackResultOneOf4
  | SourceAttackResultOneOf5
  | SourceAttackResultOneOf6
  | SourceAttackResultOneOf7
  | SourceAttackResultOneOf8;

function enumToString<T extends string>(obj: Record<T, T>): T {
    // @ts-ignore
    return Object.values(obj)[0];
}

const SourceAttackResultOneOfAttackType = enumToString(SourceAttackResultOneOfAttackTypeEnum);
const SourceAttackResultOneOf1AttackType = enumToString(SourceAttackResultOneOf1AttackTypeEnum);
const SourceAttackResultOneOf2AttackType = enumToString(SourceAttackResultOneOf2AttackTypeEnum);
const SourceAttackResultOneOf3AttackType = enumToString(SourceAttackResultOneOf3AttackTypeEnum);
const SourceAttackResultOneOf4AttackType = enumToString(SourceAttackResultOneOf4AttackTypeEnum);
const SourceAttackResultOneOf5AttackType = enumToString(SourceAttackResultOneOf5AttackTypeEnum);
const SourceAttackResultOneOf6AttackType = enumToString(SourceAttackResultOneOf6AttackTypeEnum);
const SourceAttackResultOneOf7AttackType = enumToString(SourceAttackResultOneOf7AttackTypeEnum);
const SourceAttackResultOneOf8AttackType = enumToString(SourceAttackResultOneOf8AttackTypeEnum);

export function SourceAttackResultFromJSON(json: any): SourceAttackResult {
    return SourceAttackResultFromJSONTyped(json, false);
}

export function SourceAttackResultFromJSONTyped(json: any, ignoreDiscriminator: boolean): SourceAttackResult {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    switch (json['attack_type']) {
        
        case SourceAttackResultOneOfAttackType:
            return SourceAttackResultOneOfFromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf1AttackType:
            return SourceAttackResultOneOf1FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf2AttackType:
            return SourceAttackResultOneOf2FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf3AttackType:
            return SourceAttackResultOneOf3FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf4AttackType:
            return SourceAttackResultOneOf4FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf5AttackType:
            return SourceAttackResultOneOf5FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf6AttackType:
            return SourceAttackResultOneOf6FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf7AttackType:
            return SourceAttackResultOneOf7FromJSONTyped(json, ignoreDiscriminator);
        case SourceAttackResultOneOf8AttackType:
            return SourceAttackResultOneOf8FromJSONTyped(json, ignoreDiscriminator);
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
        
        case SourceAttackResultOneOfAttackType:
            return SourceAttackResultOneOfToJSON(value);
        case SourceAttackResultOneOf1AttackType:
            return SourceAttackResultOneOf1ToJSON(value);
        case SourceAttackResultOneOf2AttackType:
            return SourceAttackResultOneOf2ToJSON(value);
        case SourceAttackResultOneOf3AttackType:
            return SourceAttackResultOneOf3ToJSON(value);
        case SourceAttackResultOneOf4AttackType:
            return SourceAttackResultOneOf4ToJSON(value);
        case SourceAttackResultOneOf5AttackType:
            return SourceAttackResultOneOf5ToJSON(value);
        case SourceAttackResultOneOf6AttackType:
            return SourceAttackResultOneOf6ToJSON(value);
        case SourceAttackResultOneOf7AttackType:
            return SourceAttackResultOneOf7ToJSON(value);
        case SourceAttackResultOneOf8AttackType:
            return SourceAttackResultOneOf8ToJSON(value);
        default:
            throw new Error("No variant of SourceAttackResult exists with 'attackType=" + value["attackType"] + "'");
    }

}
