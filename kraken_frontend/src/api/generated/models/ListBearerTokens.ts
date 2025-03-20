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
import type { FullBearerToken } from './FullBearerToken';
import {
    FullBearerTokenFromJSON,
    FullBearerTokenFromJSONTyped,
    FullBearerTokenToJSON,
} from './FullBearerToken';

/**
 * A list of bearer tokens
 * @export
 * @interface ListBearerTokens
 */
export interface ListBearerTokens {
    /**
     * List of tokens
     * @type {Array<FullBearerToken>}
     * @memberof ListBearerTokens
     */
    tokens: Array<FullBearerToken>;
}

/**
 * Check if a given object implements the ListBearerTokens interface.
 */
export function instanceOfListBearerTokens(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "tokens" in value;

    return isInstance;
}

export function ListBearerTokensFromJSON(json: any): ListBearerTokens {
    return ListBearerTokensFromJSONTyped(json, false);
}

export function ListBearerTokensFromJSONTyped(json: any, ignoreDiscriminator: boolean): ListBearerTokens {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'tokens': ((json['tokens'] as Array<any>).map(FullBearerTokenFromJSON)),
    };
}

export function ListBearerTokensToJSON(value?: ListBearerTokens | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'tokens': ((value.tokens as Array<any>).map(FullBearerTokenToJSON)),
    };
}

