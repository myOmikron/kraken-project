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
import type { SimpleBruteforceSubdomainsResult } from './SimpleBruteforceSubdomainsResult';
import {
    SimpleBruteforceSubdomainsResultFromJSON,
    SimpleBruteforceSubdomainsResultFromJSONTyped,
    SimpleBruteforceSubdomainsResultToJSON,
} from './SimpleBruteforceSubdomainsResult';

/**
 * Response containing paginated data
 * @export
 * @interface BruteforceSubdomainsResultsPage
 */
export interface BruteforceSubdomainsResultsPage {
    /**
     * The page's items
     * @type {Array<SimpleBruteforceSubdomainsResult>}
     * @memberof BruteforceSubdomainsResultsPage
     */
    items: Array<SimpleBruteforceSubdomainsResult>;
    /**
     * The limit this page was retrieved with
     * @type {number}
     * @memberof BruteforceSubdomainsResultsPage
     */
    limit: number;
    /**
     * The offset this page was retrieved with
     * @type {number}
     * @memberof BruteforceSubdomainsResultsPage
     */
    offset: number;
    /**
     * The total number of items this page is a subset of
     * @type {number}
     * @memberof BruteforceSubdomainsResultsPage
     */
    total: number;
}

/**
 * Check if a given object implements the BruteforceSubdomainsResultsPage interface.
 */
export function instanceOfBruteforceSubdomainsResultsPage(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "items" in value;
    isInstance = isInstance && "limit" in value;
    isInstance = isInstance && "offset" in value;
    isInstance = isInstance && "total" in value;

    return isInstance;
}

export function BruteforceSubdomainsResultsPageFromJSON(json: any): BruteforceSubdomainsResultsPage {
    return BruteforceSubdomainsResultsPageFromJSONTyped(json, false);
}

export function BruteforceSubdomainsResultsPageFromJSONTyped(json: any, ignoreDiscriminator: boolean): BruteforceSubdomainsResultsPage {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'items': ((json['items'] as Array<any>).map(SimpleBruteforceSubdomainsResultFromJSON)),
        'limit': json['limit'],
        'offset': json['offset'],
        'total': json['total'],
    };
}

export function BruteforceSubdomainsResultsPageToJSON(value?: BruteforceSubdomainsResultsPage | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'items': ((value.items as Array<any>).map(SimpleBruteforceSubdomainsResultToJSON)),
        'limit': value.limit,
        'offset': value.offset,
        'total': value.total,
    };
}

