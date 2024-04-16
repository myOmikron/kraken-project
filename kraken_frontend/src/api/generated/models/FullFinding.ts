/* tslint:disable */
/* eslint-disable */
/**
 * kraken
 * The core component of kraken-project
 *
 * The version of the OpenAPI document: 0.1.0
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
import type { SimpleFindingAffected } from './SimpleFindingAffected';
import {
    SimpleFindingAffectedFromJSON,
    SimpleFindingAffectedFromJSONTyped,
    SimpleFindingAffectedToJSON,
} from './SimpleFindingAffected';
import type { SimpleFindingCategory } from './SimpleFindingCategory';
import {
    SimpleFindingCategoryFromJSON,
    SimpleFindingCategoryFromJSONTyped,
    SimpleFindingCategoryToJSON,
} from './SimpleFindingCategory';
import type { SimpleFindingDefinition } from './SimpleFindingDefinition';
import {
    SimpleFindingDefinitionFromJSON,
    SimpleFindingDefinitionFromJSONTyped,
    SimpleFindingDefinitionToJSON,
} from './SimpleFindingDefinition';

/**
 * A full finding
 * @export
 * @interface FullFinding
 */
export interface FullFinding {
    /**
     * The uuid of the finding
     * @type {string}
     * @memberof FullFinding
     */
    uuid: string;
    /**
     * 
     * @type {SimpleFindingDefinition}
     * @memberof FullFinding
     */
    definition: SimpleFindingDefinition;
    /**
     * 
     * @type {FindingSeverity}
     * @memberof FullFinding
     */
    severity: FindingSeverity;
    /**
     * List of all affected objects
     * @type {Array<SimpleFindingAffected>}
     * @memberof FullFinding
     */
    affected: Array<SimpleFindingAffected>;
    /**
     * Notes about the finding provided by the user
     * 
     * May be used for documenting command invocation or other information
     * that are provided by the user
     * @type {string}
     * @memberof FullFinding
     */
    userDetails: string;
    /**
     * Details of the finding that comes from the attack module
     * 
     * This field should only be read-only for the user
     * @type {string}
     * @memberof FullFinding
     */
    toolDetails?: string | null;
    /**
     * The uuid to download a screenshot with
     * @type {string}
     * @memberof FullFinding
     */
    screenshot?: string | null;
    /**
     * The uuid to download a log file with
     * @type {string}
     * @memberof FullFinding
     */
    logFile?: string | null;
    /**
     * The point in time this finding was created
     * @type {Date}
     * @memberof FullFinding
     */
    createdAt: Date;
    /**
     * The list of categories this finding falls into
     * @type {Array<SimpleFindingCategory>}
     * @memberof FullFinding
     */
    categories: Array<SimpleFindingCategory>;
}

/**
 * Check if a given object implements the FullFinding interface.
 */
export function instanceOfFullFinding(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "definition" in value;
    isInstance = isInstance && "severity" in value;
    isInstance = isInstance && "affected" in value;
    isInstance = isInstance && "userDetails" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "categories" in value;

    return isInstance;
}

export function FullFindingFromJSON(json: any): FullFinding {
    return FullFindingFromJSONTyped(json, false);
}

export function FullFindingFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullFinding {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'definition': SimpleFindingDefinitionFromJSON(json['definition']),
        'severity': FindingSeverityFromJSON(json['severity']),
        'affected': ((json['affected'] as Array<any>).map(SimpleFindingAffectedFromJSON)),
        'userDetails': json['user_details'],
        'toolDetails': !exists(json, 'tool_details') ? undefined : json['tool_details'],
        'screenshot': !exists(json, 'screenshot') ? undefined : json['screenshot'],
        'logFile': !exists(json, 'log_file') ? undefined : json['log_file'],
        'createdAt': (new Date(json['created_at'])),
        'categories': ((json['categories'] as Array<any>).map(SimpleFindingCategoryFromJSON)),
    };
}

export function FullFindingToJSON(value?: FullFinding | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'definition': SimpleFindingDefinitionToJSON(value.definition),
        'severity': FindingSeverityToJSON(value.severity),
        'affected': ((value.affected as Array<any>).map(SimpleFindingAffectedToJSON)),
        'user_details': value.userDetails,
        'tool_details': value.toolDetails,
        'screenshot': value.screenshot,
        'log_file': value.logFile,
        'created_at': (value.createdAt.toISOString()),
        'categories': ((value.categories as Array<any>).map(SimpleFindingCategoryToJSON)),
    };
}

