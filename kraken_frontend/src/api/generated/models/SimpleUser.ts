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
 * This struct holds the user information.
 * 
 * Note that `username` is unique, but as it is changeable,
 * identify the user by its `uuid`
 * @export
 * @interface SimpleUser
 */
export interface SimpleUser {
    /**
     * The uuid of the user
     * @type {string}
     * @memberof SimpleUser
     */
    uuid: string;
    /**
     * The username of the user
     * @type {string}
     * @memberof SimpleUser
     */
    username: string;
    /**
     * The displayname of the user
     * @type {string}
     * @memberof SimpleUser
     */
    displayName: string;
}

/**
 * Check if a given object implements the SimpleUser interface.
 */
export function instanceOfSimpleUser(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "username" in value;
    isInstance = isInstance && "displayName" in value;

    return isInstance;
}

export function SimpleUserFromJSON(json: any): SimpleUser {
    return SimpleUserFromJSONTyped(json, false);
}

export function SimpleUserFromJSONTyped(json: any, ignoreDiscriminator: boolean): SimpleUser {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'username': json['username'],
        'displayName': json['display_name'],
    };
}

export function SimpleUserToJSON(value?: SimpleUser | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'username': value.username,
        'display_name': value.displayName,
    };
}

