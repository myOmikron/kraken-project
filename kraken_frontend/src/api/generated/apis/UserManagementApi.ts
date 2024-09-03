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


import * as runtime from '../runtime';
import type {
  ApiErrorResponse,
  FullUser,
  ListUsers,
  SetPasswordRequest,
  UpdateMeRequest,
} from '../models';
import {
    ApiErrorResponseFromJSON,
    ApiErrorResponseToJSON,
    FullUserFromJSON,
    FullUserToJSON,
    ListUsersFromJSON,
    ListUsersToJSON,
    SetPasswordRequestFromJSON,
    SetPasswordRequestToJSON,
    UpdateMeRequestFromJSON,
    UpdateMeRequestToJSON,
} from '../models';

export interface SetPasswordOperationRequest {
    setPasswordRequest: SetPasswordRequest;
}

export interface UpdateMeOperationRequest {
    updateMeRequest: UpdateMeRequest;
}

/**
 * 
 */
export class UserManagementApi extends runtime.BaseAPI {

    /**
     * This may be used to create invitations for workspaces
     * Request all users
     */
    async getAllUsersRaw(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<ListUsers>> {
        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/users`,
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => ListUsersFromJSON(jsonValue));
    }

    /**
     * This may be used to create invitations for workspaces
     * Request all users
     */
    async getAllUsers(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<ListUsers> {
        const response = await this.getAllUsersRaw(initOverrides);
        return await response.value();
    }

    /**
     * Retrieve the own user
     */
    async getMeRaw(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullUser>> {
        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/users/me`,
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullUserFromJSON(jsonValue));
    }

    /**
     * Retrieve the own user
     */
    async getMe(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullUser> {
        const response = await this.getMeRaw(initOverrides);
        return await response.value();
    }

    /**
     * Set a new password
     */
    async setPasswordRaw(requestParameters: SetPasswordOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.setPasswordRequest === null || requestParameters.setPasswordRequest === undefined) {
            throw new runtime.RequiredError('setPasswordRequest','Required parameter requestParameters.setPasswordRequest was null or undefined when calling setPassword.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/users/setPassword`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: SetPasswordRequestToJSON(requestParameters.setPasswordRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Set a new password
     */
    async setPassword(requestParameters: SetPasswordOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.setPasswordRaw(requestParameters, initOverrides);
    }

    /**
     * All parameters are optional, but at least one of them must be supplied.
     * Updates the own user
     */
    async updateMeRaw(requestParameters: UpdateMeOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.updateMeRequest === null || requestParameters.updateMeRequest === undefined) {
            throw new runtime.RequiredError('updateMeRequest','Required parameter requestParameters.updateMeRequest was null or undefined when calling updateMe.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/users/me`,
            method: 'PUT',
            headers: headerParameters,
            query: queryParameters,
            body: UpdateMeRequestToJSON(requestParameters.updateMeRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * All parameters are optional, but at least one of them must be supplied.
     * Updates the own user
     */
    async updateMe(requestParameters: UpdateMeOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.updateMeRaw(requestParameters, initOverrides);
    }

}
