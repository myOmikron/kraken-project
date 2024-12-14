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
 * A simple representation of a service detection result
 * @export
 * @interface FullServiceDetectionResult
 */
export interface FullServiceDetectionResult {
    /**
     * The primary key
     * @type {string}
     * @memberof FullServiceDetectionResult
     */
    uuid: string;
    /**
     * The attack which produced this result
     * @type {string}
     * @memberof FullServiceDetectionResult
     */
    attack: string;
    /**
     * The point in time, this result was produced
     * @type {Date}
     * @memberof FullServiceDetectionResult
     */
    createdAt: Date;
    /**
     * The certainty a service is detected
     * @type {string}
     * @memberof FullServiceDetectionResult
     */
    certainty: FullServiceDetectionResultCertaintyEnum;
    /**
     * The found names of the service
     * @type {Array<string>}
     * @memberof FullServiceDetectionResult
     */
    serviceNames: Array<string>;
    /**
     * The ip address a port was found on
     * @type {string}
     * @memberof FullServiceDetectionResult
     */
    host: string;
    /**
     * Port number
     * @type {number}
     * @memberof FullServiceDetectionResult
     */
    port: number;
}


/**
 * @export
 */
export const FullServiceDetectionResultCertaintyEnum = {
    Historical: 'Historical',
    SupposedTo: 'SupposedTo',
    MaybeVerified: 'MaybeVerified',
    DefinitelyVerified: 'DefinitelyVerified',
    UnknownService: 'UnknownService'
} as const;
export type FullServiceDetectionResultCertaintyEnum = typeof FullServiceDetectionResultCertaintyEnum[keyof typeof FullServiceDetectionResultCertaintyEnum];


/**
 * Check if a given object implements the FullServiceDetectionResult interface.
 */
export function instanceOfFullServiceDetectionResult(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "attack" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "certainty" in value;
    isInstance = isInstance && "serviceNames" in value;
    isInstance = isInstance && "host" in value;
    isInstance = isInstance && "port" in value;

    return isInstance;
}

export function FullServiceDetectionResultFromJSON(json: any): FullServiceDetectionResult {
    return FullServiceDetectionResultFromJSONTyped(json, false);
}

export function FullServiceDetectionResultFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullServiceDetectionResult {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'attack': json['attack'],
        'createdAt': (new Date(json['created_at'])),
        'certainty': json['certainty'],
        'serviceNames': json['service_names'],
        'host': json['host'],
        'port': json['port'],
    };
}

export function FullServiceDetectionResultToJSON(value?: FullServiceDetectionResult | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'attack': value.attack,
        'created_at': (value.createdAt.toISOString()),
        'certainty': value.certainty,
        'service_names': value.serviceNames,
        'host': value.host,
        'port': value.port,
    };
}
