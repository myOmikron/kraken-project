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
import type { SimpleDomain } from './SimpleDomain';
import {
    SimpleDomainFromJSON,
    SimpleDomainFromJSONTyped,
    SimpleDomainToJSON,
} from './SimpleDomain';

/**
 * The response with all domains in a workspace
 * @export
 * @interface GetAllDomainsResponse
 */
export interface GetAllDomainsResponse {
    /**
     * 
     * @type {Array<SimpleDomain>}
     * @memberof GetAllDomainsResponse
     */
    domains: Array<SimpleDomain>;
}

/**
 * Check if a given object implements the GetAllDomainsResponse interface.
 */
export function instanceOfGetAllDomainsResponse(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "domains" in value;

    return isInstance;
}

export function GetAllDomainsResponseFromJSON(json: any): GetAllDomainsResponse {
    return GetAllDomainsResponseFromJSONTyped(json, false);
}

export function GetAllDomainsResponseFromJSONTyped(json: any, ignoreDiscriminator: boolean): GetAllDomainsResponse {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'domains': ((json['domains'] as Array<any>).map(SimpleDomainFromJSON)),
    };
}

export function GetAllDomainsResponseToJSON(value?: GetAllDomainsResponse | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'domains': ((value.domains as Array<any>).map(SimpleDomainToJSON)),
    };
}
