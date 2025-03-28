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
import type { DnsTxtScanEntryOneOf1Spf } from './DnsTxtScanEntryOneOf1Spf';
import {
    DnsTxtScanEntryOneOf1SpfFromJSON,
    DnsTxtScanEntryOneOf1SpfFromJSONTyped,
    DnsTxtScanEntryOneOf1SpfToJSON,
} from './DnsTxtScanEntryOneOf1Spf';

/**
 * 
 * @export
 * @interface DnsTxtScanEntryOneOf1
 */
export interface DnsTxtScanEntryOneOf1 {
    /**
     * 
     * @type {DnsTxtScanEntryOneOf1Spf}
     * @memberof DnsTxtScanEntryOneOf1
     */
    spf: DnsTxtScanEntryOneOf1Spf;
}

/**
 * Check if a given object implements the DnsTxtScanEntryOneOf1 interface.
 */
export function instanceOfDnsTxtScanEntryOneOf1(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "spf" in value;

    return isInstance;
}

export function DnsTxtScanEntryOneOf1FromJSON(json: any): DnsTxtScanEntryOneOf1 {
    return DnsTxtScanEntryOneOf1FromJSONTyped(json, false);
}

export function DnsTxtScanEntryOneOf1FromJSONTyped(json: any, ignoreDiscriminator: boolean): DnsTxtScanEntryOneOf1 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'spf': DnsTxtScanEntryOneOf1SpfFromJSON(json['Spf']),
    };
}

export function DnsTxtScanEntryOneOf1ToJSON(value?: DnsTxtScanEntryOneOf1 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'Spf': DnsTxtScanEntryOneOf1SpfToJSON(value.spf),
    };
}

