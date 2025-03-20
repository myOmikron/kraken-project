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
import type { FindingAffectedObject } from './FindingAffectedObject';
import {
    FindingAffectedObjectFromJSON,
    FindingAffectedObjectFromJSONTyped,
    FindingAffectedObjectToJSON,
} from './FindingAffectedObject';
import type { FullFinding } from './FullFinding';
import {
    FullFindingFromJSON,
    FullFindingFromJSONTyped,
    FullFindingToJSON,
} from './FullFinding';
import type { SimpleTag } from './SimpleTag';
import {
    SimpleTagFromJSON,
    SimpleTagFromJSONTyped,
    SimpleTagToJSON,
} from './SimpleTag';

/**
 * An affected object's details and the finding it is affected by
 * @export
 * @interface FullFindingAffected
 */
export interface FullFindingAffected {
    /**
     * 
     * @type {FullFinding}
     * @memberof FullFindingAffected
     */
    finding: FullFinding;
    /**
     * 
     * @type {FindingAffectedObject}
     * @memberof FullFindingAffected
     */
    affected: FindingAffectedObject;
    /**
     * List of tags for the affected object
     * @type {Array<SimpleTag>}
     * @memberof FullFindingAffected
     */
    affectedTags: Array<SimpleTag>;
    /**
     * Notes about the finding included in the data export
     * 
     * May be used for documenting details about the finding
     * used to generate reports outside of kraken.
     * @type {string}
     * @memberof FullFindingAffected
     */
    exportDetails: string;
    /**
     * Notes about the finding provided by the user
     * 
     * May be used for documenting command invocation or other information
     * that are provided by the user
     * @type {string}
     * @memberof FullFindingAffected
     */
    userDetails: string;
    /**
     * Details of the finding that comes from the attack module
     * 
     * This field should only be read-only for the user
     * @type {string}
     * @memberof FullFindingAffected
     */
    toolDetails?: string | null;
    /**
     * The uuid to download a screenshot with
     * @type {string}
     * @memberof FullFindingAffected
     */
    screenshot?: string | null;
    /**
     * The uuid to download a log file with
     * @type {string}
     * @memberof FullFindingAffected
     */
    logFile?: string | null;
    /**
     * The point in time this object was attached to the finding
     * @type {Date}
     * @memberof FullFindingAffected
     */
    createdAt: Date;
}

/**
 * Check if a given object implements the FullFindingAffected interface.
 */
export function instanceOfFullFindingAffected(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "finding" in value;
    isInstance = isInstance && "affected" in value;
    isInstance = isInstance && "affectedTags" in value;
    isInstance = isInstance && "exportDetails" in value;
    isInstance = isInstance && "userDetails" in value;
    isInstance = isInstance && "createdAt" in value;

    return isInstance;
}

export function FullFindingAffectedFromJSON(json: any): FullFindingAffected {
    return FullFindingAffectedFromJSONTyped(json, false);
}

export function FullFindingAffectedFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullFindingAffected {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'finding': FullFindingFromJSON(json['finding']),
        'affected': FindingAffectedObjectFromJSON(json['affected']),
        'affectedTags': ((json['affected_tags'] as Array<any>).map(SimpleTagFromJSON)),
        'exportDetails': json['export_details'],
        'userDetails': json['user_details'],
        'toolDetails': !exists(json, 'tool_details') ? undefined : json['tool_details'],
        'screenshot': !exists(json, 'screenshot') ? undefined : json['screenshot'],
        'logFile': !exists(json, 'log_file') ? undefined : json['log_file'],
        'createdAt': (new Date(json['created_at'])),
    };
}

export function FullFindingAffectedToJSON(value?: FullFindingAffected | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'finding': FullFindingToJSON(value.finding),
        'affected': FindingAffectedObjectToJSON(value.affected),
        'affected_tags': ((value.affectedTags as Array<any>).map(SimpleTagToJSON)),
        'export_details': value.exportDetails,
        'user_details': value.userDetails,
        'tool_details': value.toolDetails,
        'screenshot': value.screenshot,
        'log_file': value.logFile,
        'created_at': (value.createdAt.toISOString()),
    };
}

