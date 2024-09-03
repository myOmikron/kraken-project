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
/**
 * The request to set a new password for a user
 * @export
 * @interface SetPasswordRequest
 */
export interface SetPasswordRequest {
    /**
     * The current password
     * @type {string}
     * @memberof SetPasswordRequest
     */
    currentPassword: string;
    /**
     * The new password
     * @type {string}
     * @memberof SetPasswordRequest
     */
    newPassword: string;
}

/**
 * Check if a given object implements the SetPasswordRequest interface.
 */
export function instanceOfSetPasswordRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "currentPassword" in value;
    isInstance = isInstance && "newPassword" in value;

    return isInstance;
}

export function SetPasswordRequestFromJSON(json: any): SetPasswordRequest {
    return SetPasswordRequestFromJSONTyped(json, false);
}

export function SetPasswordRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): SetPasswordRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'currentPassword': json['current_password'],
        'newPassword': json['new_password'],
    };
}

export function SetPasswordRequestToJSON(value?: SetPasswordRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'current_password': value.currentPassword,
        'new_password': value.newPassword,
    };
}

