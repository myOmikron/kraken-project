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
import type { ManualHostCertainty } from './ManualHostCertainty';
import {
    ManualHostCertaintyFromJSON,
    ManualHostCertaintyFromJSONTyped,
    ManualHostCertaintyToJSON,
} from './ManualHostCertainty';
import type { OsType } from './OsType';
import {
    OsTypeFromJSON,
    OsTypeFromJSONTyped,
    OsTypeToJSON,
} from './OsType';
import type { SimpleUser } from './SimpleUser';
import {
    SimpleUserFromJSON,
    SimpleUserFromJSONTyped,
    SimpleUserToJSON,
} from './SimpleUser';

/**
 * A manually inserted host
 * @export
 * @interface ManualInsertOneOf1
 */
export interface ManualInsertOneOf1 {
    /**
     * The host's ip address
     * @type {string}
     * @memberof ManualInsertOneOf1
     */
    ipAddr: string;
    /**
     * 
     * @type {OsType}
     * @memberof ManualInsertOneOf1
     */
    osType: OsType;
    /**
     * 
     * @type {ManualHostCertainty}
     * @memberof ManualInsertOneOf1
     */
    certainty: ManualHostCertainty;
    /**
     * 
     * @type {SimpleUser}
     * @memberof ManualInsertOneOf1
     */
    user: SimpleUser;
    /**
     * The workspace the host was inserted to
     * @type {string}
     * @memberof ManualInsertOneOf1
     */
    workspace: string;
    /**
     * The point in time, the host was inserted
     * @type {Date}
     * @memberof ManualInsertOneOf1
     */
    createdAt: Date;
    /**
     * 
     * @type {string}
     * @memberof ManualInsertOneOf1
     */
    type: ManualInsertOneOf1TypeEnum;
}


/**
 * @export
 */
export const ManualInsertOneOf1TypeEnum = {
    Host: 'Host'
} as const;
export type ManualInsertOneOf1TypeEnum = typeof ManualInsertOneOf1TypeEnum[keyof typeof ManualInsertOneOf1TypeEnum];


/**
 * Check if a given object implements the ManualInsertOneOf1 interface.
 */
export function instanceOfManualInsertOneOf1(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "ipAddr" in value;
    isInstance = isInstance && "osType" in value;
    isInstance = isInstance && "certainty" in value;
    isInstance = isInstance && "user" in value;
    isInstance = isInstance && "workspace" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function ManualInsertOneOf1FromJSON(json: any): ManualInsertOneOf1 {
    return ManualInsertOneOf1FromJSONTyped(json, false);
}

export function ManualInsertOneOf1FromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualInsertOneOf1 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'ipAddr': json['ip_addr'],
        'osType': OsTypeFromJSON(json['os_type']),
        'certainty': ManualHostCertaintyFromJSON(json['certainty']),
        'user': SimpleUserFromJSON(json['user']),
        'workspace': json['workspace'],
        'createdAt': (new Date(json['created_at'])),
        'type': json['type'],
    };
}

export function ManualInsertOneOf1ToJSON(value?: ManualInsertOneOf1 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'ip_addr': value.ipAddr,
        'os_type': OsTypeToJSON(value.osType),
        'certainty': ManualHostCertaintyToJSON(value.certainty),
        'user': SimpleUserToJSON(value.user),
        'workspace': value.workspace,
        'created_at': (value.createdAt.toISOString()),
        'type': value.type,
    };
}

