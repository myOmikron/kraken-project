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
 * The port's protocol is [`PortProtocol::Sctp`]
 * @export
 * @interface ServiceProtocolsOneOf3Sctp
 */
export interface ServiceProtocolsOneOf3Sctp {
    /**
     * The service responds to raw sctp
     * @type {boolean}
     * @memberof ServiceProtocolsOneOf3Sctp
     */
    raw: boolean;
}

/**
 * Check if a given object implements the ServiceProtocolsOneOf3Sctp interface.
 */
export function instanceOfServiceProtocolsOneOf3Sctp(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "raw" in value;

    return isInstance;
}

export function ServiceProtocolsOneOf3SctpFromJSON(json: any): ServiceProtocolsOneOf3Sctp {
    return ServiceProtocolsOneOf3SctpFromJSONTyped(json, false);
}

export function ServiceProtocolsOneOf3SctpFromJSONTyped(json: any, ignoreDiscriminator: boolean): ServiceProtocolsOneOf3Sctp {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'raw': json['raw'],
    };
}

export function ServiceProtocolsOneOf3SctpToJSON(value?: ServiceProtocolsOneOf3Sctp | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'raw': value.raw,
    };
}

