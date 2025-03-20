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
import type { Color } from './Color';
import {
    ColorFromJSON,
    ColorFromJSONTyped,
    ColorToJSON,
} from './Color';

/**
 * The request to update a workspace tag
 * @export
 * @interface UpdateWorkspaceTag
 */
export interface UpdateWorkspaceTag {
    /**
     * Name of the tag
     * @type {string}
     * @memberof UpdateWorkspaceTag
     */
    name?: string | null;
    /**
     * 
     * @type {Color}
     * @memberof UpdateWorkspaceTag
     */
    color?: Color | null;
}

/**
 * Check if a given object implements the UpdateWorkspaceTag interface.
 */
export function instanceOfUpdateWorkspaceTag(value: object): boolean {
    let isInstance = true;

    return isInstance;
}

export function UpdateWorkspaceTagFromJSON(json: any): UpdateWorkspaceTag {
    return UpdateWorkspaceTagFromJSONTyped(json, false);
}

export function UpdateWorkspaceTagFromJSONTyped(json: any, ignoreDiscriminator: boolean): UpdateWorkspaceTag {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'name': !exists(json, 'name') ? undefined : json['name'],
        'color': !exists(json, 'color') ? undefined : ColorFromJSON(json['color']),
    };
}

export function UpdateWorkspaceTagToJSON(value?: UpdateWorkspaceTag | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'name': value.name,
        'color': ColorToJSON(value.color),
    };
}

