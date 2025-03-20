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
 * Query parameters for filtering the ports to get
 * @export
 * @interface GetAllPortsQuery
 */
export interface GetAllPortsQuery {
    /**
     * Number of items to retrieve
     * @type {number}
     * @memberof GetAllPortsQuery
     */
    limit: number;
    /**
     * Position in the whole list to start retrieving from
     * @type {number}
     * @memberof GetAllPortsQuery
     */
    offset: number;
    /**
     * Only get ports associated with a specific host
     * @type {string}
     * @memberof GetAllPortsQuery
     */
    host?: string | null;
    /**
     * An optional general filter to apply
     * @type {string}
     * @memberof GetAllPortsQuery
     */
    globalFilter?: string | null;
    /**
     * An optional port specific filter to apply
     * @type {string}
     * @memberof GetAllPortsQuery
     */
    portFilter?: string | null;
}

/**
 * Check if a given object implements the GetAllPortsQuery interface.
 */
export function instanceOfGetAllPortsQuery(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "limit" in value;
    isInstance = isInstance && "offset" in value;

    return isInstance;
}

export function GetAllPortsQueryFromJSON(json: any): GetAllPortsQuery {
    return GetAllPortsQueryFromJSONTyped(json, false);
}

export function GetAllPortsQueryFromJSONTyped(json: any, ignoreDiscriminator: boolean): GetAllPortsQuery {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'limit': json['limit'],
        'offset': json['offset'],
        'host': !exists(json, 'host') ? undefined : json['host'],
        'globalFilter': !exists(json, 'global_filter') ? undefined : json['global_filter'],
        'portFilter': !exists(json, 'port_filter') ? undefined : json['port_filter'],
    };
}

export function GetAllPortsQueryToJSON(value?: GetAllPortsQuery | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'limit': value.limit,
        'offset': value.offset,
        'host': value.host,
        'global_filter': value.globalFilter,
        'port_filter': value.portFilter,
    };
}

