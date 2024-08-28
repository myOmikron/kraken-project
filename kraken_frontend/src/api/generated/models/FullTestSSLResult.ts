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
import type { TestSSLFinding } from './TestSSLFinding';
import {
    TestSSLFindingFromJSON,
    TestSSLFindingFromJSONTyped,
    TestSSLFindingToJSON,
} from './TestSSLFinding';

/**
 * The results of a `testssl.sh` scan
 * @export
 * @interface FullTestSSLResult
 */
export interface FullTestSSLResult {
    /**
     * The primary key
     * @type {string}
     * @memberof FullTestSSLResult
     */
    uuid: string;
    /**
     * The attack which produced this result
     * @type {string}
     * @memberof FullTestSSLResult
     */
    attack: string;
    /**
     * The point in time, this result was produced
     * @type {Date}
     * @memberof FullTestSSLResult
     */
    createdAt: Date;
    /**
     * The domain which was used for SNI and certificate verification
     * @type {string}
     * @memberof FullTestSSLResult
     */
    domain?: string | null;
    /**
     * The scanned ip address
     * @type {string}
     * @memberof FullTestSSLResult
     */
    ip: string;
    /**
     * The scanned port
     * @type {number}
     * @memberof FullTestSSLResult
     */
    port: number;
    /**
     * The ip address' rDNS name
     * @type {string}
     * @memberof FullTestSSLResult
     */
    rdns: string;
    /**
     * The detected service
     * @type {string}
     * @memberof FullTestSSLResult
     */
    service: string;
    /**
     * The scan's findings
     * 
     * This includes, log messages, extracted information (for example cert parameters) and tests for vulnerabilities / bad options.
     * @type {Array<TestSSLFinding>}
     * @memberof FullTestSSLResult
     */
    findings: Array<TestSSLFinding>;
}

/**
 * Check if a given object implements the FullTestSSLResult interface.
 */
export function instanceOfFullTestSSLResult(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "attack" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "ip" in value;
    isInstance = isInstance && "port" in value;
    isInstance = isInstance && "rdns" in value;
    isInstance = isInstance && "service" in value;
    isInstance = isInstance && "findings" in value;

    return isInstance;
}

export function FullTestSSLResultFromJSON(json: any): FullTestSSLResult {
    return FullTestSSLResultFromJSONTyped(json, false);
}

export function FullTestSSLResultFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullTestSSLResult {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'attack': json['attack'],
        'createdAt': (new Date(json['created_at'])),
        'domain': !exists(json, 'domain') ? undefined : json['domain'],
        'ip': json['ip'],
        'port': json['port'],
        'rdns': json['rdns'],
        'service': json['service'],
        'findings': ((json['findings'] as Array<any>).map(TestSSLFindingFromJSON)),
    };
}

export function FullTestSSLResultToJSON(value?: FullTestSSLResult | null): any {
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
        'domain': value.domain,
        'ip': value.ip,
        'port': value.port,
        'rdns': value.rdns,
        'service': value.service,
        'findings': ((value.findings as Array<any>).map(TestSSLFindingToJSON)),
    };
}

