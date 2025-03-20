/* tslint:disable */
/* eslint-disable */
/**
 * kraken
 * The core component of kraken-project
 *
 * The version of the OpenAPI document: 0.5.0
 * Contact: git@omikron.dev
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { SimpleAttack } from './SimpleAttack';
import {
    SimpleAttackFromJSON,
    SimpleAttackFromJSONTyped,
    SimpleAttackToJSON,
} from './SimpleAttack';

/**
 * A notification about a started attack
 * @export
 * @interface WsMessageOneOf2
 */
export interface WsMessageOneOf2 {
    /**
     * 
     * @type {SimpleAttack}
     * @memberof WsMessageOneOf2
     */
    attack: SimpleAttack;
    /**
     * 
     * @type {string}
     * @memberof WsMessageOneOf2
     */
    type: WsMessageOneOf2TypeEnum;
}


/**
 * @export
 */
export const WsMessageOneOf2TypeEnum = {
    AttackStarted: 'AttackStarted'
} as const;
export type WsMessageOneOf2TypeEnum = typeof WsMessageOneOf2TypeEnum[keyof typeof WsMessageOneOf2TypeEnum];


/**
 * Check if a given object implements the WsMessageOneOf2 interface.
 */
export function instanceOfWsMessageOneOf2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "attack" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function WsMessageOneOf2FromJSON(json: any): WsMessageOneOf2 {
    return WsMessageOneOf2FromJSONTyped(json, false);
}

export function WsMessageOneOf2FromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessageOneOf2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'attack': SimpleAttackFromJSON(json['attack']),
        'type': json['type'],
    };
}

export function WsMessageOneOf2ToJSON(value?: WsMessageOneOf2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'attack': SimpleAttackToJSON(value.attack),
        'type': value.type,
    };
}

