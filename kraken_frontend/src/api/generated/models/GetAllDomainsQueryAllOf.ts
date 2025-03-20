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
/**
 * 
 * @export
 * @interface GetAllDomainsQueryAllOf
 */
export interface GetAllDomainsQueryAllOf {
    /**
     * Only get domains pointing to a specific host
     * 
     * This includes domains which point to another domain which points to this host.
     * @type {string}
     * @memberof GetAllDomainsQueryAllOf
     */
    host?: string | null;
    /**
     * An optional general filter to apply
     * @type {string}
     * @memberof GetAllDomainsQueryAllOf
     */
    globalFilter?: string | null;
    /**
     * An optional domain specific filter to apply
     * @type {string}
     * @memberof GetAllDomainsQueryAllOf
     */
    domainFilter?: string | null;
}

/**
 * Check if a given object implements the GetAllDomainsQueryAllOf interface.
 */
export function instanceOfGetAllDomainsQueryAllOf(value: object): boolean {
    let isInstance = true;

    return isInstance;
}

export function GetAllDomainsQueryAllOfFromJSON(json: any): GetAllDomainsQueryAllOf {
    return GetAllDomainsQueryAllOfFromJSONTyped(json, false);
}

export function GetAllDomainsQueryAllOfFromJSONTyped(json: any, ignoreDiscriminator: boolean): GetAllDomainsQueryAllOf {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'host': !exists(json, 'host') ? undefined : json['host'],
        'globalFilter': !exists(json, 'global_filter') ? undefined : json['global_filter'],
        'domainFilter': !exists(json, 'domain_filter') ? undefined : json['domain_filter'],
    };
}

export function GetAllDomainsQueryAllOfToJSON(value?: GetAllDomainsQueryAllOf | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'host': value.host,
        'global_filter': value.globalFilter,
        'domain_filter': value.domainFilter,
    };
}

