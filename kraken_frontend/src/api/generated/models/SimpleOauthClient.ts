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
 * A simple (secret-less) version of a workspace
 * @export
 * @interface SimpleOauthClient
 */
export interface SimpleOauthClient {
    /**
     * The uuid of the client
     * @type {string}
     * @memberof SimpleOauthClient
     */
    uuid: string;
    /**
     * The name of the client
     * @type {string}
     * @memberof SimpleOauthClient
     */
    name: string;
    /**
     * The redirect url of the client
     * @type {string}
     * @memberof SimpleOauthClient
     */
    redirectUri: string;
}

/**
 * Check if a given object implements the SimpleOauthClient interface.
 */
export function instanceOfSimpleOauthClient(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "name" in value;
    isInstance = isInstance && "redirectUri" in value;

    return isInstance;
}

export function SimpleOauthClientFromJSON(json: any): SimpleOauthClient {
    return SimpleOauthClientFromJSONTyped(json, false);
}

export function SimpleOauthClientFromJSONTyped(json: any, ignoreDiscriminator: boolean): SimpleOauthClient {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'name': json['name'],
        'redirectUri': json['redirect_uri'],
    };
}

export function SimpleOauthClientToJSON(value?: SimpleOauthClient | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'name': value.name,
        'redirect_uri': value.redirectUri,
    };
}

