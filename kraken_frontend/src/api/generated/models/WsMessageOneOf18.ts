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
/**
 * A port was deleted
 * @export
 * @interface WsMessageOneOf18
 */
export interface WsMessageOneOf18 {
    /**
     * The workspace this port is related to
     * @type {string}
     * @memberof WsMessageOneOf18
     */
    workspace: string;
    /**
     * The uuid of the deleted port
     * @type {string}
     * @memberof WsMessageOneOf18
     */
    port: string;
    /**
     * 
     * @type {string}
     * @memberof WsMessageOneOf18
     */
    type: WsMessageOneOf18TypeEnum;
}


/**
 * @export
 */
export const WsMessageOneOf18TypeEnum = {
    DeletedPort: 'DeletedPort'
} as const;
export type WsMessageOneOf18TypeEnum = typeof WsMessageOneOf18TypeEnum[keyof typeof WsMessageOneOf18TypeEnum];


/**
 * Check if a given object implements the WsMessageOneOf18 interface.
 */
export function instanceOfWsMessageOneOf18(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "workspace" in value;
    isInstance = isInstance && "port" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function WsMessageOneOf18FromJSON(json: any): WsMessageOneOf18 {
    return WsMessageOneOf18FromJSONTyped(json, false);
}

export function WsMessageOneOf18FromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessageOneOf18 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'workspace': json['workspace'],
        'port': json['port'],
        'type': json['type'],
    };
}

export function WsMessageOneOf18ToJSON(value?: WsMessageOneOf18 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'workspace': value.workspace,
        'port': value.port,
        'type': value.type,
    };
}

