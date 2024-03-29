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
import type { UpdateFindingDefinitionRequest } from './UpdateFindingDefinitionRequest';
import {
    UpdateFindingDefinitionRequestFromJSON,
    UpdateFindingDefinitionRequestFromJSONTyped,
    UpdateFindingDefinitionRequestToJSON,
} from './UpdateFindingDefinitionRequest';

/**
 * A finding definition has been updated
 * @export
 * @interface WsMessageOneOf25
 */
export interface WsMessageOneOf25 {
    /**
     * The uuid of the finding definition
     * @type {string}
     * @memberof WsMessageOneOf25
     */
    uuid: string;
    /**
     * 
     * @type {UpdateFindingDefinitionRequest}
     * @memberof WsMessageOneOf25
     */
    update: UpdateFindingDefinitionRequest;
    /**
     * 
     * @type {string}
     * @memberof WsMessageOneOf25
     */
    type: WsMessageOneOf25TypeEnum;
}


/**
 * @export
 */
export const WsMessageOneOf25TypeEnum = {
    UpdatedFindingDefinition: 'UpdatedFindingDefinition'
} as const;
export type WsMessageOneOf25TypeEnum = typeof WsMessageOneOf25TypeEnum[keyof typeof WsMessageOneOf25TypeEnum];


/**
 * Check if a given object implements the WsMessageOneOf25 interface.
 */
export function instanceOfWsMessageOneOf25(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "update" in value;
    isInstance = isInstance && "type" in value;

    return isInstance;
}

export function WsMessageOneOf25FromJSON(json: any): WsMessageOneOf25 {
    return WsMessageOneOf25FromJSONTyped(json, false);
}

export function WsMessageOneOf25FromJSONTyped(json: any, ignoreDiscriminator: boolean): WsMessageOneOf25 {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'update': UpdateFindingDefinitionRequestFromJSON(json['update']),
        'type': json['type'],
    };
}

export function WsMessageOneOf25ToJSON(value?: WsMessageOneOf25 | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'update': UpdateFindingDefinitionRequestToJSON(value.update),
        'type': value.type,
    };
}

