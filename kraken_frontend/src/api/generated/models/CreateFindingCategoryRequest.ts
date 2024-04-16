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
import type { Color } from './Color';
import {
    ColorFromJSON,
    ColorFromJSONTyped,
    ColorToJSON,
} from './Color';

/**
 * The request to create a finding category
 * @export
 * @interface CreateFindingCategoryRequest
 */
export interface CreateFindingCategoryRequest {
    /**
     * The category's name
     * @type {string}
     * @memberof CreateFindingCategoryRequest
     */
    name: string;
    /**
     * 
     * @type {Color}
     * @memberof CreateFindingCategoryRequest
     */
    color: Color;
}

/**
 * Check if a given object implements the CreateFindingCategoryRequest interface.
 */
export function instanceOfCreateFindingCategoryRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "name" in value;
    isInstance = isInstance && "color" in value;

    return isInstance;
}

export function CreateFindingCategoryRequestFromJSON(json: any): CreateFindingCategoryRequest {
    return CreateFindingCategoryRequestFromJSONTyped(json, false);
}

export function CreateFindingCategoryRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): CreateFindingCategoryRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'name': json['name'],
        'color': ColorFromJSON(json['color']),
    };
}

export function CreateFindingCategoryRequestToJSON(value?: CreateFindingCategoryRequest | null): any {
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

