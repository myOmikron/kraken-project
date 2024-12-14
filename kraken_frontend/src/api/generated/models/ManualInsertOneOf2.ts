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
import type { ManualPortCertainty } from './ManualPortCertainty';
import {
    ManualPortCertaintyFromJSON,
    ManualPortCertaintyFromJSONTyped,
    ManualPortCertaintyToJSON,
} from './ManualPortCertainty';
import type { PortProtocol } from './PortProtocol';
import {
    PortProtocolFromJSON,
    PortProtocolFromJSONTyped,
    PortProtocolToJSON,
} from './PortProtocol';
import type { SimpleUser } from './SimpleUser';
import {
    SimpleUserFromJSON,
    SimpleUserFromJSONTyped,
    SimpleUserToJSON,
} from './SimpleUser';

/**
 * A manually inserted port
 * @export
 * @interface ManualInsertOneOf2
 */
export interface ManualInsertOneOf2 {
    /**
     * The inserted port
     * @type {number}
     * @memberof ManualInsertOneOf2
     */
    port: number;
    /**
     * 
     * @type {PortProtocol}
     * @memberof ManualInsertOneOf2
     */
    protocol: PortProtocol;
    /**
     * 
     * @type {ManualPortCertainty}
     * @memberof ManualInsertOneOf2
     */
    certainty: ManualPortCertainty;
    /**
     * The host's ip address
     * @type {string}
     * @memberof ManualInsertOneOf2
     */
    host: string;
    /**
     * 
     * @type {SimpleUser}
     * @memberof ManualInsertOneOf2
     */
    user: SimpleUser;
    /**
     * The workspace the port was inserted to
     * @type {string}
     * @memberof ManualInsertOneOf2
     */
    workspace: string;
    /**
     * The point in time, the port was inserted
     * @type {Date}
     * @memberof ManualInsertOneOf2
     */
    createdAt: Date;
    /**
     * 
     * @type {string}
     * @memberof ManualInsertOneOf2
     */
    type: ManualInsertOneOf2TypeEnum;
}


/**
 * @export
 */
export const ManualInsertOneOf2TypeEnum = {
    Port: 'Port'
} as const;
export type ManualInsertOneOf2TypeEnum = typeof ManualInsertOneOf2TypeEnum[keyof typeof ManualInsertOneOf2TypeEnum];


/**
 * Check if a given object implements the ManualInsertOneOf2 interface.
 */
export function instanceOfManualInsertOneOf2(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "port" in value;
    isInstance = isInstance && "protocol" in value;
    isInstance = isInstance && "certainty" in value;
    isInstance = isInstance && "host" in value;
    isInstance = isInstance && "user" in value;
    isInstance = isInstance && "workspace" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function ManualInsertOneOf2FromJSON(json: any): ManualInsertOneOf2 {
    return ManualInsertOneOf2FromJSONTyped(json, false);
}

export function ManualInsertOneOf2FromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualInsertOneOf2 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'port': json['port'],
        'protocol': PortProtocolFromJSON(json['protocol']),
        'certainty': ManualPortCertaintyFromJSON(json['certainty']),
        'host': json['host'],
        'user': SimpleUserFromJSON(json['user']),
        'workspace': json['workspace'],
        'createdAt': (new Date(json['created_at'])),
        'type': json['type'],
    };
}

export function ManualInsertOneOf2ToJSON(value?: ManualInsertOneOf2 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'port': value.port,
        'protocol': PortProtocolToJSON(value.protocol),
        'certainty': ManualPortCertaintyToJSON(value.certainty),
        'host': value.host,
        'user': SimpleUserToJSON(value.user),
        'workspace': value.workspace,
        'created_at': (value.createdAt.toISOString()),
        'type': value.type,
    };
}
