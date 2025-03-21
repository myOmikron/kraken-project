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
import type { FindingSeverity } from './FindingSeverity';
import {
    FindingSeverityFromJSON,
    FindingSeverityFromJSONTyped,
    FindingSeverityToJSON,
} from './FindingSeverity';
import type { SimpleFindingCategory } from './SimpleFindingCategory';
import {
    SimpleFindingCategoryFromJSON,
    SimpleFindingCategoryFromJSONTyped,
    SimpleFindingCategoryToJSON,
} from './SimpleFindingCategory';

/**
 * The simple definition of a finding
 * @export
 * @interface SimpleFindingDefinition
 */
export interface SimpleFindingDefinition {
    /**
     * The uuid of a finding definition
     * @type {string}
     * @memberof SimpleFindingDefinition
     */
    uuid: string;
    /**
     * Name of the new finding definition
     * @type {string}
     * @memberof SimpleFindingDefinition
     */
    name: string;
    /**
     * CVE of the finding definition
     * @type {string}
     * @memberof SimpleFindingDefinition
     */
    cve?: string | null;
    /**
     * 
     * @type {FindingSeverity}
     * @memberof SimpleFindingDefinition
     */
    severity: FindingSeverity;
    /**
     * Short summary of the finding
     * @type {string}
     * @memberof SimpleFindingDefinition
     */
    summary: string;
    /**
     * Expected time duration required for the remediation
     * @type {string}
     * @memberof SimpleFindingDefinition
     */
    remediationDuration: string;
    /**
     * The point in time this finding definition was created
     * @type {Date}
     * @memberof SimpleFindingDefinition
     */
    createdAt: Date;
    /**
     * The list of categories this finding falls into
     * @type {Array<SimpleFindingCategory>}
     * @memberof SimpleFindingDefinition
     */
    categories: Array<SimpleFindingCategory>;
}

/**
 * Check if a given object implements the SimpleFindingDefinition interface.
 */
export function instanceOfSimpleFindingDefinition(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "name" in value;
    isInstance = isInstance && "severity" in value;
    isInstance = isInstance && "summary" in value;
    isInstance = isInstance && "remediationDuration" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "categories" in value;

    return isInstance;
}

export function SimpleFindingDefinitionFromJSON(json: any): SimpleFindingDefinition {
    return SimpleFindingDefinitionFromJSONTyped(json, false);
}

export function SimpleFindingDefinitionFromJSONTyped(json: any, ignoreDiscriminator: boolean): SimpleFindingDefinition {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'name': json['name'],
        'cve': !exists(json, 'cve') ? undefined : json['cve'],
        'severity': FindingSeverityFromJSON(json['severity']),
        'summary': json['summary'],
        'remediationDuration': json['remediation_duration'],
        'createdAt': (new Date(json['created_at'])),
        'categories': ((json['categories'] as Array<any>).map(SimpleFindingCategoryFromJSON)),
    };
}

export function SimpleFindingDefinitionToJSON(value?: SimpleFindingDefinition | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'name': value.name,
        'cve': value.cve,
        'severity': FindingSeverityToJSON(value.severity),
        'summary': value.summary,
        'remediation_duration': value.remediationDuration,
        'created_at': (value.createdAt.toISOString()),
        'categories': ((value.categories as Array<any>).map(SimpleFindingCategoryToJSON)),
    };
}

