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
/**
 * Defines this position of a cursor
 * @export
 * @interface CursorPosition
 */
export interface CursorPosition {
    /**
     * The line the cursor was placed in
     * @type {number}
     * @memberof CursorPosition
     */
    line: number;
    /**
     * The column the cursor was placed in
     * @type {number}
     * @memberof CursorPosition
     */
    column: number;
}

/**
 * Check if a given object implements the CursorPosition interface.
 */
export function instanceOfCursorPosition(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "line" in value;
    isInstance = isInstance && "column" in value;

    return isInstance;
}

export function CursorPositionFromJSON(json: any): CursorPosition {
    return CursorPositionFromJSONTyped(json, false);
}

export function CursorPositionFromJSONTyped(json: any, ignoreDiscriminator: boolean): CursorPosition {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'line': json['line'],
        'column': json['column'],
    };
}

export function CursorPositionToJSON(value?: CursorPosition | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'line': value.line,
        'column': value.column,
    };
}

