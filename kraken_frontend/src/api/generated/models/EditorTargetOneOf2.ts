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
import type { EditorTargetOneOf2FindingAffected } from './EditorTargetOneOf2FindingAffected';
import {
    EditorTargetOneOf2FindingAffectedFromJSON,
    EditorTargetOneOf2FindingAffectedFromJSONTyped,
    EditorTargetOneOf2FindingAffectedToJSON,
} from './EditorTargetOneOf2FindingAffected';

/**
 * 
 * @export
 * @interface EditorTargetOneOf2
 */
export interface EditorTargetOneOf2 {
    /**
     * 
     * @type {EditorTargetOneOf2FindingAffected}
     * @memberof EditorTargetOneOf2
     */
    findingAffected: EditorTargetOneOf2FindingAffected;
}

/**
 * Check if a given object implements the EditorTargetOneOf2 interface.
 */
export function instanceOfEditorTargetOneOf2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "findingAffected" in value;

    return isInstance;
}

export function EditorTargetOneOf2FromJSON(json: any): EditorTargetOneOf2 {
    return EditorTargetOneOf2FromJSONTyped(json, false);
}

export function EditorTargetOneOf2FromJSONTyped(json: any, ignoreDiscriminator: boolean): EditorTargetOneOf2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'findingAffected': EditorTargetOneOf2FindingAffectedFromJSON(json['FindingAffected']),
    };
}

export function EditorTargetOneOf2ToJSON(value?: EditorTargetOneOf2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'FindingAffected': EditorTargetOneOf2FindingAffectedToJSON(value.findingAffected),
    };
}
