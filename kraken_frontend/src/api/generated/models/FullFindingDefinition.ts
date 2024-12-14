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
 * The full definition of a finding
 * @export
 * @interface FullFindingDefinition
 */
export interface FullFindingDefinition {
    /**
     * The uuid of a finding definition
     * @type {string}
     * @memberof FullFindingDefinition
     */
    uuid: string;
    /**
     * Name of the new finding definition
     * @type {string}
     * @memberof FullFindingDefinition
     */
    name: string;
    /**
     * 
     * @type {FindingSeverity}
     * @memberof FullFindingDefinition
     */
    severity: FindingSeverity;
    /**
     * Short summary of the finding
     * @type {string}
     * @memberof FullFindingDefinition
     */
    summary: string;
    /**
     * Optional linked CVE
     * @type {string}
     * @memberof FullFindingDefinition
     */
    cve?: string | null;
    /**
     * The full description of the finding
     * @type {string}
     * @memberof FullFindingDefinition
     */
    description: string;
    /**
     * The impact of the finding in general.
     * @type {string}
     * @memberof FullFindingDefinition
     */
    impact: string;
    /**
     * How to remediate the finding
     * @type {string}
     * @memberof FullFindingDefinition
     */
    remediation: string;
    /**
     * Any references to get more information about the definition of a finding.
     * @type {string}
     * @memberof FullFindingDefinition
     */
    references: string;
    /**
     * The point in time this finding definition was created
     * @type {Date}
     * @memberof FullFindingDefinition
     */
    createdAt: Date;
    /**
     * The list of categories this finding definition falls into
     * @type {Array<SimpleFindingCategory>}
     * @memberof FullFindingDefinition
     */
    categories: Array<SimpleFindingCategory>;
}

/**
 * Check if a given object implements the FullFindingDefinition interface.
 */
export function instanceOfFullFindingDefinition(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "name" in value;
    isInstance = isInstance && "severity" in value;
    isInstance = isInstance && "summary" in value;
    isInstance = isInstance && "description" in value;
    isInstance = isInstance && "impact" in value;
    isInstance = isInstance && "remediation" in value;
    isInstance = isInstance && "references" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "categories" in value;

    return isInstance;
}

export function FullFindingDefinitionFromJSON(json: any): FullFindingDefinition {
    return FullFindingDefinitionFromJSONTyped(json, false);
}

export function FullFindingDefinitionFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullFindingDefinition {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'name': json['name'],
        'severity': FindingSeverityFromJSON(json['severity']),
        'summary': json['summary'],
        'cve': !exists(json, 'cve') ? undefined : json['cve'],
        'description': json['description'],
        'impact': json['impact'],
        'remediation': json['remediation'],
        'references': json['references'],
        'createdAt': (new Date(json['created_at'])),
        'categories': ((json['categories'] as Array<any>).map(SimpleFindingCategoryFromJSON)),
    };
}

export function FullFindingDefinitionToJSON(value?: FullFindingDefinition | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'name': value.name,
        'severity': FindingSeverityToJSON(value.severity),
        'summary': value.summary,
        'cve': value.cve,
        'description': value.description,
        'impact': value.impact,
        'remediation': value.remediation,
        'references': value.references,
        'created_at': (value.createdAt.toISOString()),
        'categories': ((value.categories as Array<any>).map(SimpleFindingCategoryToJSON)),
    };
}
