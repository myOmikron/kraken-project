/* tslint:disable */
/* eslint-disable */
/**
 * kraken
 * The core component of kraken-project
 *
 * The version of the OpenAPI document: 0.1.0
 * Contact: git@omikron.dev
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

import { exists, mapValues } from '../runtime';
import type { SimpleTcpPortScanResult } from './SimpleTcpPortScanResult';
import {
    SimpleTcpPortScanResultFromJSON,
    SimpleTcpPortScanResultFromJSONTyped,
    SimpleTcpPortScanResultToJSON,
} from './SimpleTcpPortScanResult';

/**
 * 
 * @export
 * @interface SourceAttackResultOneOf1
 */
export interface SourceAttackResultOneOf1 {
    /**
     * 
     * @type {string}
     * @memberof SourceAttackResultOneOf1
     */
    attackType: SourceAttackResultOneOf1AttackTypeEnum;
    /**
     * The [`AttackType::TcpPortScan`] and its results
     * @type {Array<SimpleTcpPortScanResult>}
     * @memberof SourceAttackResultOneOf1
     */
    results: Array<SimpleTcpPortScanResult>;
}


/**
 * @export
 */
export const SourceAttackResultOneOf1AttackTypeEnum = {
    TcpPortScan: 'TcpPortScan'
} as const;
export type SourceAttackResultOneOf1AttackTypeEnum = typeof SourceAttackResultOneOf1AttackTypeEnum[keyof typeof SourceAttackResultOneOf1AttackTypeEnum];


/**
 * Check if a given object implements the SourceAttackResultOneOf1 interface.
 */
export function instanceOfSourceAttackResultOneOf1(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "attackType" in value;
    isInstance = isInstance && "results" in value;

    return isInstance;
}

export function SourceAttackResultOneOf1FromJSON(json: any): SourceAttackResultOneOf1 {
    return SourceAttackResultOneOf1FromJSONTyped(json, false);
}

export function SourceAttackResultOneOf1FromJSONTyped(json: any, ignoreDiscriminator: boolean): SourceAttackResultOneOf1 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'attackType': json['attack_type'],
        'results': ((json['results'] as Array<any>).map(SimpleTcpPortScanResultFromJSON)),
    };
}

export function SourceAttackResultOneOf1ToJSON(value?: SourceAttackResultOneOf1 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'attack_type': value.attackType,
        'results': ((value.results as Array<any>).map(SimpleTcpPortScanResultToJSON)),
    };
}
