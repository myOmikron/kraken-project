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
import type { ServiceProtocolsOneOf1Tcp } from './ServiceProtocolsOneOf1Tcp';
import {
    ServiceProtocolsOneOf1TcpFromJSON,
    ServiceProtocolsOneOf1TcpFromJSONTyped,
    ServiceProtocolsOneOf1TcpToJSON,
} from './ServiceProtocolsOneOf1Tcp';

/**
 * 
 * @export
 * @interface ServiceProtocolsOneOf1
 */
export interface ServiceProtocolsOneOf1 {
    /**
     * 
     * @type {ServiceProtocolsOneOf1Tcp}
     * @memberof ServiceProtocolsOneOf1
     */
    tcp: ServiceProtocolsOneOf1Tcp;
}

/**
 * Check if a given object implements the ServiceProtocolsOneOf1 interface.
 */
export function instanceOfServiceProtocolsOneOf1(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "tcp" in value;

    return isInstance;
}

export function ServiceProtocolsOneOf1FromJSON(json: any): ServiceProtocolsOneOf1 {
    return ServiceProtocolsOneOf1FromJSONTyped(json, false);
}

export function ServiceProtocolsOneOf1FromJSONTyped(json: any, ignoreDiscriminator: boolean): ServiceProtocolsOneOf1 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'tcp': ServiceProtocolsOneOf1TcpFromJSON(json['Tcp']),
    };
}

export function ServiceProtocolsOneOf1ToJSON(value?: ServiceProtocolsOneOf1 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'Tcp': ServiceProtocolsOneOf1TcpToJSON(value.tcp),
    };
}

