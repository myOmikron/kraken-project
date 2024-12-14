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
import type { FullDnsTxtScanResult } from './FullDnsTxtScanResult';
import {
    FullDnsTxtScanResultFromJSON,
    FullDnsTxtScanResultFromJSONTyped,
    FullDnsTxtScanResultToJSON,
} from './FullDnsTxtScanResult';

/**
 * Response containing paginated data
 * @export
 * @interface DnsTxtScanResultsPage
 */
export interface DnsTxtScanResultsPage {
    /**
     * The page's items
     * @type {Array<FullDnsTxtScanResult>}
     * @memberof DnsTxtScanResultsPage
     */
    items: Array<FullDnsTxtScanResult>;
    /**
     * The limit this page was retrieved with
     * @type {number}
     * @memberof DnsTxtScanResultsPage
     */
    limit: number;
    /**
     * The offset this page was retrieved with
     * @type {number}
     * @memberof DnsTxtScanResultsPage
     */
    offset: number;
    /**
     * The total number of items this page is a subset of
     * @type {number}
     * @memberof DnsTxtScanResultsPage
     */
    total: number;
}

/**
 * Check if a given object implements the DnsTxtScanResultsPage interface.
 */
export function instanceOfDnsTxtScanResultsPage(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "items" in value;
    isInstance = isInstance && "limit" in value;
    isInstance = isInstance && "offset" in value;
    isInstance = isInstance && "total" in value;

    return isInstance;
}

export function DnsTxtScanResultsPageFromJSON(json: any): DnsTxtScanResultsPage {
    return DnsTxtScanResultsPageFromJSONTyped(json, false);
}

export function DnsTxtScanResultsPageFromJSONTyped(json: any, ignoreDiscriminator: boolean): DnsTxtScanResultsPage {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'items': ((json['items'] as Array<any>).map(FullDnsTxtScanResultFromJSON)),
        'limit': json['limit'],
        'offset': json['offset'],
        'total': json['total'],
    };
}

export function DnsTxtScanResultsPageToJSON(value?: DnsTxtScanResultsPage | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'items': ((value.items as Array<any>).map(FullDnsTxtScanResultToJSON)),
        'limit': value.limit,
        'offset': value.offset,
        'total': value.total,
    };
}
