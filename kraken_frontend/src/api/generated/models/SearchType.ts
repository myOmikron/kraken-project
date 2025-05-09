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

import {
    SearchTypeOneOf,
    instanceOfSearchTypeOneOf,
    SearchTypeOneOfFromJSON,
    SearchTypeOneOfFromJSONTyped,
    SearchTypeOneOfToJSON,
} from './SearchTypeOneOf';
import {
    SearchTypeOneOf1,
    instanceOfSearchTypeOneOf1,
    SearchTypeOneOf1FromJSON,
    SearchTypeOneOf1FromJSONTyped,
    SearchTypeOneOf1ToJSON,
} from './SearchTypeOneOf1';
import {
    SearchTypeOneOf2,
    instanceOfSearchTypeOneOf2,
    SearchTypeOneOf2FromJSON,
    SearchTypeOneOf2FromJSONTyped,
    SearchTypeOneOf2ToJSON,
} from './SearchTypeOneOf2';
import {
    SearchTypeOneOf3,
    instanceOfSearchTypeOneOf3,
    SearchTypeOneOf3FromJSON,
    SearchTypeOneOf3FromJSONTyped,
    SearchTypeOneOf3ToJSON,
} from './SearchTypeOneOf3';
import {
    SearchTypeOneOf4,
    instanceOfSearchTypeOneOf4,
    SearchTypeOneOf4FromJSON,
    SearchTypeOneOf4FromJSONTyped,
    SearchTypeOneOf4ToJSON,
} from './SearchTypeOneOf4';

/**
 * @type SearchType
 * A specific search type
 * @export
 */
export type SearchType = SearchTypeOneOf | SearchTypeOneOf1 | SearchTypeOneOf2 | SearchTypeOneOf3 | SearchTypeOneOf4;

export function SearchTypeFromJSON(json: any): SearchType {
    return SearchTypeFromJSONTyped(json, false);
}

export function SearchTypeFromJSONTyped(json: any, ignoreDiscriminator: boolean): SearchType {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return { ...SearchTypeOneOfFromJSONTyped(json, true), ...SearchTypeOneOf1FromJSONTyped(json, true), ...SearchTypeOneOf2FromJSONTyped(json, true), ...SearchTypeOneOf3FromJSONTyped(json, true), ...SearchTypeOneOf4FromJSONTyped(json, true) };
}

export function SearchTypeToJSON(value?: SearchType | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }

    if (instanceOfSearchTypeOneOf(value)) {
        return SearchTypeOneOfToJSON(value as SearchTypeOneOf);
    }
    if (instanceOfSearchTypeOneOf1(value)) {
        return SearchTypeOneOf1ToJSON(value as SearchTypeOneOf1);
    }
    if (instanceOfSearchTypeOneOf2(value)) {
        return SearchTypeOneOf2ToJSON(value as SearchTypeOneOf2);
    }
    if (instanceOfSearchTypeOneOf3(value)) {
        return SearchTypeOneOf3ToJSON(value as SearchTypeOneOf3);
    }
    if (instanceOfSearchTypeOneOf4(value)) {
        return SearchTypeOneOf4ToJSON(value as SearchTypeOneOf4);
    }

    return {};
}

