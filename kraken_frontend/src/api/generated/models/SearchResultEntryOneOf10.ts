/* tslint:disable */
/* eslint-disable */
/**
 * kraken
 * The core component of kraken-project
 *
 * The version of the OpenAPI document: 0.4.2
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
 * @interface SearchResultEntryOneOf10
 */
export interface SearchResultEntryOneOf10 {
    /**
     * 
     * @type {FullUdpServiceDetectionResult}
     * @memberof SearchResultEntryOneOf10
     */
    udpServiceDetectionResult: FullUdpServiceDetectionResult;
}

/**
 * Check if a given object implements the SearchResultEntryOneOf10 interface.
 */
export function instanceOfSearchResultEntryOneOf10(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "udpServiceDetectionResult" in value;

    return isInstance;
}

export function SearchResultEntryOneOf10FromJSON(json: any): SearchResultEntryOneOf10 {
    return SearchResultEntryOneOf10FromJSONTyped(json, false);
}

export function SearchResultEntryOneOf10FromJSONTyped(json: any, ignoreDiscriminator: boolean): SearchResultEntryOneOf10 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'udpServiceDetectionResult': FullUdpServiceDetectionResultFromJSON(json['UdpServiceDetectionResult']),
    };
}

export function SearchResultEntryOneOf10ToJSON(value?: SearchResultEntryOneOf10 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'UdpServiceDetectionResult': FullUdpServiceDetectionResultToJSON(value.udpServiceDetectionResult),
    };
}
