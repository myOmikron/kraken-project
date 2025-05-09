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
import type { DnsTxtScanEntry } from './DnsTxtScanEntry';
import {
    DnsTxtScanEntryFromJSON,
    DnsTxtScanEntryFromJSONTyped,
    DnsTxtScanEntryToJSON,
} from './DnsTxtScanEntry';
import type { DnsTxtScanSummaryType } from './DnsTxtScanSummaryType';
import {
    DnsTxtScanSummaryTypeFromJSON,
    DnsTxtScanSummaryTypeFromJSONTyped,
    DnsTxtScanSummaryTypeToJSON,
} from './DnsTxtScanSummaryType';

/**
 * The full representation of a dns txt scan result
 * @export
 * @interface FullDnsTxtScanResult
 */
export interface FullDnsTxtScanResult {
    /**
     * The primary key
     * @type {string}
     * @memberof FullDnsTxtScanResult
     */
    uuid: string;
    /**
     * The attack which produced this result
     * @type {string}
     * @memberof FullDnsTxtScanResult
     */
    attack: string;
    /**
     * The point in time, this result was produced
     * @type {Date}
     * @memberof FullDnsTxtScanResult
     */
    createdAt: Date;
    /**
     * The source address
     * @type {string}
     * @memberof FullDnsTxtScanResult
     */
    domain: string;
    /**
     * 
     * @type {DnsTxtScanSummaryType}
     * @memberof FullDnsTxtScanResult
     */
    collectionType: DnsTxtScanSummaryType;
    /**
     * List of result entries. The kind depends on the `collection_type` in this object.
     * @type {Array<DnsTxtScanEntry>}
     * @memberof FullDnsTxtScanResult
     */
    entries: Array<DnsTxtScanEntry>;
}

/**
 * Check if a given object implements the FullDnsTxtScanResult interface.
 */
export function instanceOfFullDnsTxtScanResult(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "attack" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "domain" in value;
    isInstance = isInstance && "collectionType" in value;
    isInstance = isInstance && "entries" in value;

    return isInstance;
}

export function FullDnsTxtScanResultFromJSON(json: any): FullDnsTxtScanResult {
    return FullDnsTxtScanResultFromJSONTyped(json, false);
}

export function FullDnsTxtScanResultFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullDnsTxtScanResult {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'attack': json['attack'],
        'createdAt': (new Date(json['created_at'])),
        'domain': json['domain'],
        'collectionType': DnsTxtScanSummaryTypeFromJSON(json['collection_type']),
        'entries': ((json['entries'] as Array<any>).map(DnsTxtScanEntryFromJSON)),
    };
}

export function FullDnsTxtScanResultToJSON(value?: FullDnsTxtScanResult | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'attack': value.attack,
        'created_at': (value.createdAt.toISOString()),
        'domain': value.domain,
        'collection_type': DnsTxtScanSummaryTypeToJSON(value.collectionType),
        'entries': ((value.entries as Array<any>).map(DnsTxtScanEntryToJSON)),
    };
}

