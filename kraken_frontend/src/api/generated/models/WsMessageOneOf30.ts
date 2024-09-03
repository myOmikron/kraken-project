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
import type { CursorPosition } from './CursorPosition';
import {
    CursorPositionFromJSON,
    CursorPositionFromJSONTyped,
    CursorPositionToJSON,
} from './CursorPosition';
import type { EditorTarget } from './EditorTarget';
import {
    EditorTargetFromJSON,
    EditorTargetFromJSONTyped,
    EditorTargetToJSON,
} from './EditorTarget';
import type { SimpleUser } from './SimpleUser';
import {
    SimpleUserFromJSON,
    SimpleUserFromJSONTyped,
    SimpleUserToJSON,
} from './SimpleUser';

/**
 * A user has changed its cursor position in an editor
 * @export
 * @interface WsMessageOneOf30
 */
export interface WsMessageOneOf30 {
    /**
     * 
     * @type {SimpleUser}
     * @memberof WsMessageOneOf30
     */
    user: SimpleUser;
    /**
     * 
     * @type {EditorTarget}
     * @memberof WsMessageOneOf30
     */
    target: EditorTarget;
    /**
     * 
     * @type {CursorPosition}
     * @memberof WsMessageOneOf30
     */
    cursor: CursorPosition;
    /**
     * 
     * @type {string}
     * @memberof WsMessageOneOf30
     */
    type: WsMessageOneOf30TypeEnum;
}


/**
 * @export
 */
export const WsMessageOneOf30TypeEnum = {
    EditorChangedCursor: 'EditorChangedCursor'
} as const;
export type WsMessageOneOf30TypeEnum = typeof WsMessageOneOf30TypeEnum[keyof typeof WsMessageOneOf30TypeEnum];


/**
 * Check if a given object implements the WsMessageOneOf30 interface.
 */
export function instanceOfWsMessageOneOf30(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "user" in value;
    isInstance = isInstance && "target" in value;
    isInstance = isInstance && "cursor" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function WsMessageOneOf30FromJSON(json: any): WsMessageOneOf30 {
    return WsMessageOneOf30FromJSONTyped(json, false);
}

export function WsMessageOneOf30FromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessageOneOf30 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'user': SimpleUserFromJSON(json['user']),
        'target': EditorTargetFromJSON(json['target']),
        'cursor': CursorPositionFromJSON(json['cursor']),
        'type': json['type'],
    };
}

export function WsMessageOneOf30ToJSON(value?: WsMessageOneOf30 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'user': SimpleUserToJSON(value.user),
        'target': EditorTargetToJSON(value.target),
        'cursor': CursorPositionToJSON(value.cursor),
        'type': value.type,
    };
}

