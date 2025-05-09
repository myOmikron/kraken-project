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
import type { FindingDefinitionUsage } from './FindingDefinitionUsage';
import {
    FindingDefinitionUsageFromJSON,
    FindingDefinitionUsageFromJSONTyped,
    FindingDefinitionUsageToJSON,
} from './FindingDefinitionUsage';

/**
 * A list of findings using a specific finding definition
 * @export
 * @interface ListFindingDefinitionUsages
 */
export interface ListFindingDefinitionUsages {
    /**
     * A list of findings using a specific finding definition
     * @type {Array<FindingDefinitionUsage>}
     * @memberof ListFindingDefinitionUsages
     */
    usages: Array<FindingDefinitionUsage>;
}

/**
 * Check if a given object implements the ListFindingDefinitionUsages interface.
 */
export function instanceOfListFindingDefinitionUsages(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "usages" in value;

    return isInstance;
}

export function ListFindingDefinitionUsagesFromJSON(json: any): ListFindingDefinitionUsages {
    return ListFindingDefinitionUsagesFromJSONTyped(json, false);
}

export function ListFindingDefinitionUsagesFromJSONTyped(json: any, ignoreDiscriminator: boolean): ListFindingDefinitionUsages {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'usages': ((json['usages'] as Array<any>).map(FindingDefinitionUsageFromJSON)),
    };
}

export function ListFindingDefinitionUsagesToJSON(value?: ListFindingDefinitionUsages | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'usages': ((value.usages as Array<any>).map(FindingDefinitionUsageToJSON)),
    };
}

