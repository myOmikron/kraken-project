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
import type { AggregationType } from './AggregationType';
import {
    AggregationTypeFromJSON,
    AggregationTypeFromJSONTyped,
    AggregationTypeToJSON,
} from './AggregationType';

/**
 * Global tags were updated on an aggregation
 * @export
 * @interface WsMessageOneOf21
 */
export interface WsMessageOneOf21 {
    /**
     * The workspace the aggregation is related to
     * @type {string}
     * @memberof WsMessageOneOf21
     */
    workspace: string;
    /**
     * 
     * @type {AggregationType}
     * @memberof WsMessageOneOf21
     */
    aggregation: AggregationType;
    /**
     * The uuid of the model
     * @type {string}
     * @memberof WsMessageOneOf21
     */
    uuid: string;
    /**
     * The updated list of tags
     * @type {Array<string>}
     * @memberof WsMessageOneOf21
     */
    tags: Array<string>;
    /**
     * 
     * @type {string}
     * @memberof WsMessageOneOf21
     */
    type: WsMessageOneOf21TypeEnum;
}


/**
 * @export
 */
export const WsMessageOneOf21TypeEnum = {
    UpdatedGlobalTags: 'UpdatedGlobalTags'
} as const;
export type WsMessageOneOf21TypeEnum = typeof WsMessageOneOf21TypeEnum[keyof typeof WsMessageOneOf21TypeEnum];


/**
 * Check if a given object implements the WsMessageOneOf21 interface.
 */
export function instanceOfWsMessageOneOf21(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "workspace" in value;
    isInstance = isInstance && "aggregation" in value;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "tags" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function WsMessageOneOf21FromJSON(json: any): WsMessageOneOf21 {
    return WsMessageOneOf21FromJSONTyped(json, false);
}

export function WsMessageOneOf21FromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessageOneOf21 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'workspace': json['workspace'],
        'aggregation': AggregationTypeFromJSON(json['aggregation']),
        'uuid': json['uuid'],
        'tags': json['tags'],
        'type': json['type'],
    };
}

export function WsMessageOneOf21ToJSON(value?: WsMessageOneOf21 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'workspace': value.workspace,
        'aggregation': AggregationTypeToJSON(value.aggregation),
        'uuid': value.uuid,
        'tags': value.tags,
        'type': value.type,
    };
}

