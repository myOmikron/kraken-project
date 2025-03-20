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
import type { FindingDetails } from './FindingDetails';
import {
    FindingDetailsFromJSON,
    FindingDetailsFromJSONTyped,
    FindingDetailsToJSON,
} from './FindingDetails';

/**
 * The editor for the `user_details` in [Finding]
 * @export
 * @interface EditorTargetOneOf1Finding
 */
export interface EditorTargetOneOf1Finding {
    /**
     * Uuid of the [Finding]
     * @type {string}
     * @memberof EditorTargetOneOf1Finding
     */
    finding: string;
    /**
     * 
     * @type {FindingDetails}
     * @memberof EditorTargetOneOf1Finding
     */
    findingDetails: FindingDetails;
}

/**
 * Check if a given object implements the EditorTargetOneOf1Finding interface.
 */
export function instanceOfEditorTargetOneOf1Finding(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "finding" in value;
    isInstance = isInstance && "findingDetails" in value;

    return isInstance;
}

export function EditorTargetOneOf1FindingFromJSON(json: any): EditorTargetOneOf1Finding {
    return EditorTargetOneOf1FindingFromJSONTyped(json, false);
}

export function EditorTargetOneOf1FindingFromJSONTyped(json: any, ignoreDiscriminator: boolean): EditorTargetOneOf1Finding {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'finding': json['finding'],
        'findingDetails': FindingDetailsFromJSON(json['finding_details']),
    };
}

export function EditorTargetOneOf1FindingToJSON(value?: EditorTargetOneOf1Finding | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'finding': value.finding,
        'finding_details': FindingDetailsToJSON(value.findingDetails),
    };
}

