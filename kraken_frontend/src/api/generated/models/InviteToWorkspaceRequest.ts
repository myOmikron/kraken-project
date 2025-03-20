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
 * The request to invite a user to the workspace
 * @export
 * @interface InviteToWorkspaceRequest
 */
export interface InviteToWorkspaceRequest {
    /**
     * The user to invite
     * @type {string}
     * @memberof InviteToWorkspaceRequest
     */
    user: string;
}

/**
 * Check if a given object implements the InviteToWorkspaceRequest interface.
 */
export function instanceOfInviteToWorkspaceRequest(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "user" in value;

    return isInstance;
}

export function InviteToWorkspaceRequestFromJSON(json: any): InviteToWorkspaceRequest {
    return InviteToWorkspaceRequestFromJSONTyped(json, false);
}

export function InviteToWorkspaceRequestFromJSONTyped(json: any, ignoreDiscriminator: boolean): InviteToWorkspaceRequest {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'user': json['user'],
    };
}

export function InviteToWorkspaceRequestToJSON(value?: InviteToWorkspaceRequest | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'user': value.user,
    };
}

