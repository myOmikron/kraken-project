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
import type { FullUdpServiceDetectionResult } from './FullUdpServiceDetectionResult';
import {
    FullUdpServiceDetectionResultFromJSON,
    FullUdpServiceDetectionResultFromJSONTyped,
    FullUdpServiceDetectionResultToJSON,
} from './FullUdpServiceDetectionResult';

/**
 * 
 * @export
 * @interface SourceAttackResultOneOf5
 */
export interface SourceAttackResultOneOf5 {
    /**
     * 
     * @type {string}
     * @memberof SourceAttackResultOneOf5
     */
    attackType: SourceAttackResultOneOf5AttackTypeEnum;
    /**
     * The [`AttackType::UdpServiceDetection`] and its results
     * @type {Array<FullUdpServiceDetectionResult>}
     * @memberof SourceAttackResultOneOf5
     */
    results: Array<FullUdpServiceDetectionResult>;
}


/**
 * @export
 */
export const SourceAttackResultOneOf5AttackTypeEnum = {
    UdpServiceDetection: 'UdpServiceDetection'
} as const;
export type SourceAttackResultOneOf5AttackTypeEnum = typeof SourceAttackResultOneOf5AttackTypeEnum[keyof typeof SourceAttackResultOneOf5AttackTypeEnum];


/**
 * Check if a given object implements the SourceAttackResultOneOf5 interface.
 */
export function instanceOfSourceAttackResultOneOf5(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "attackType" in value;
    isInstance = isInstance && "results" in value;

    return isInstance;
}

export function SourceAttackResultOneOf5FromJSON(json: any): SourceAttackResultOneOf5 {
    return SourceAttackResultOneOf5FromJSONTyped(json, false);
}

export function SourceAttackResultOneOf5FromJSONTyped(json: any, ignoreDiscriminator: boolean): SourceAttackResultOneOf5 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'attackType': json['attack_type'],
        'results': ((json['results'] as Array<any>).map(FullUdpServiceDetectionResultFromJSON)),
    };
}

export function SourceAttackResultOneOf5ToJSON(value?: SourceAttackResultOneOf5 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'attack_type': value.attackType,
        'results': ((value.results as Array<any>).map(FullUdpServiceDetectionResultToJSON)),
    };
}

