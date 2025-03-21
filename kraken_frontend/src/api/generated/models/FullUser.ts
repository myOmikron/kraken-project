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
import type { UserPermission } from './UserPermission';
import {
    UserPermissionFromJSON,
    UserPermissionFromJSONTyped,
    UserPermissionToJSON,
} from './UserPermission';

/**
 * A single user representation
 * @export
 * @interface FullUser
 */
export interface FullUser {
    /**
     * The uuid of the user
     * @type {string}
     * @memberof FullUser
     */
    uuid: string;
    /**
     * The username of the user
     * @type {string}
     * @memberof FullUser
     */
    username: string;
    /**
     * The displayname of the user
     * @type {string}
     * @memberof FullUser
     */
    displayName: string;
    /**
     * 
     * @type {UserPermission}
     * @memberof FullUser
     */
    permission: UserPermission;
    /**
     * The point in time this user was created
     * @type {Date}
     * @memberof FullUser
     */
    createdAt: Date;
    /**
     * The last point in time when the user has logged in
     * @type {Date}
     * @memberof FullUser
     */
    lastLogin?: Date | null;
}

/**
 * Check if a given object implements the FullUser interface.
 */
export function instanceOfFullUser(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "username" in value;
    isInstance = isInstance && "displayName" in value;
    isInstance = isInstance && "permission" in value;
    isInstance = isInstance && "createdAt" in value;

    return isInstance;
}

export function FullUserFromJSON(json: any): FullUser {
    return FullUserFromJSONTyped(json, false);
}

export function FullUserFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullUser {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'username': json['username'],
        'displayName': json['display_name'],
        'permission': UserPermissionFromJSON(json['permission']),
        'createdAt': (new Date(json['created_at'])),
        'lastLogin': !exists(json, 'last_login') ? undefined : (json['last_login'] === null ? null : new Date(json['last_login'])),
    };
}

export function FullUserToJSON(value?: FullUser | null): any {
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
        'permission': UserPermissionToJSON(value.permission),
        'created_at': (value.createdAt.toISOString()),
        'last_login': value.lastLogin === undefined ? undefined : (value.lastLogin === null ? null : value.lastLogin.toISOString()),
    };
}

