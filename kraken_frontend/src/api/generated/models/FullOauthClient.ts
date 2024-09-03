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
 * A complete version of a workspace
 * @export
 * @interface FullOauthClient
 */
export interface FullOauthClient {
    /**
     * The uuid of the client
     * @type {string}
     * @memberof FullOauthClient
     */
    uuid: string;
    /**
     * The name of the client
     * @type {string}
     * @memberof FullOauthClient
     */
    name: string;
    /**
     * The redirect url of the client
     * @type {string}
     * @memberof FullOauthClient
     */
    redirectUri: string;
    /**
     * The secret of the client
     * @type {string}
     * @memberof FullOauthClient
     */
    secret: string;
}

/**
 * Check if a given object implements the FullOauthClient interface.
 */
export function instanceOfFullOauthClient(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "name" in value;
    isInstance = isInstance && "redirectUri" in value;
    isInstance = isInstance && "secret" in value;

    return isInstance;
}

export function FullOauthClientFromJSON(json: any): FullOauthClient {
    return FullOauthClientFromJSONTyped(json, false);
}

export function FullOauthClientFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullOauthClient {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'name': json['name'],
        'redirectUri': json['redirect_uri'],
        'secret': json['secret'],
    };
}

export function FullOauthClientToJSON(value?: FullOauthClient | null): any {
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
        'secret': value.secret,
    };
}

