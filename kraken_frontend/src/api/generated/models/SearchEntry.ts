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
/**
 * Searched entry
 * @export
 * @interface SearchEntry
 */
export interface SearchEntry {
    /**
     * The uuid of the search
     * @type {string}
     * @memberof SearchEntry
     */
    uuid: string;
    /**
     * The point in time this search was created
     * @type {Date}
     * @memberof SearchEntry
     */
    createdAt: Date;
    /**
     * The point in time this search was finished
     * @type {Date}
     * @memberof SearchEntry
     */
    finishedAt?: Date | null;
    /**
     * The search term that was used
     * @type {string}
     * @memberof SearchEntry
     */
    searchTerm: string;
}

/**
 * Check if a given object implements the SearchEntry interface.
 */
export function instanceOfSearchEntry(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "searchTerm" in value;

    return isInstance;
}

export function SearchEntryFromJSON(json: any): SearchEntry {
    return SearchEntryFromJSONTyped(json, false);
}

export function SearchEntryFromJSONTyped(json: any, ignoreDiscriminator: boolean): SearchEntry {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'createdAt': (new Date(json['created_at'])),
        'finishedAt': !exists(json, 'finished_at') ? undefined : (json['finished_at'] === null ? null : new Date(json['finished_at'])),
        'searchTerm': json['search_term'],
    };
}

export function SearchEntryToJSON(value?: SearchEntry | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'created_at': (value.createdAt.toISOString()),
        'finished_at': value.finishedAt === undefined ? undefined : (value.finishedAt === null ? null : value.finishedAt.toISOString()),
        'search_term': value.searchTerm,
    };
}

