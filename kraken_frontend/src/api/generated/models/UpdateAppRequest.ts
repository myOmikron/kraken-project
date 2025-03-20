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
 * Update an oauth application
 * @export
 * @interface UpdateAppRequest
 */
export interface UpdateAppRequest {
    /**
     * The name of the application
     * @type {string}
     * @memberof UpdateAppRequest
     */
    name?: string | null;
    /**
     * The redirect url of the application
     * @type {string}
     * @memberof UpdateAppRequest
     */
    redirectUri?: string | null;
}

/**
 * Check if a given object implements the UpdateAppRequest interface.
 */
export function instanceOfUpdateAppRequest(value: object): boolean {
    let isInstance = true;

    return isInstance;
}

export function UpdateAppRequestFromJSON(json: any): UpdateAppRequest {
    return UpdateAppRequestFromJSONTyped(json, false);
}

export function UpdateAppRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): UpdateAppRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'name': !exists(json, 'name') ? undefined : json['name'],
        'redirectUri': !exists(json, 'redirect_uri') ? undefined : json['redirect_uri'],
    };
}

export function UpdateAppRequestToJSON(value?: UpdateAppRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'name': value.name,
        'redirect_uri': value.redirectUri,
    };
}

