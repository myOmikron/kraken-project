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


import * as runtime from '../runtime';
import type {
  ApiErrorResponse,
  CreateUserRequest,
  FullUser,
  ListFullUsers,
  UuidResponse,
} from '../models';
import {
    ApiErrorResponseFromJSON,
    ApiErrorResponseToJSON,
    CreateUserRequestFromJSON,
    CreateUserRequestToJSON,
    FullUserFromJSON,
    FullUserToJSON,
    ListFullUsersFromJSON,
    ListFullUsersToJSON,
    UuidResponseFromJSON,
    UuidResponseToJSON,
} from '../models';

export interface CreateUserOperationRequest {
    createUserRequest: CreateUserRequest;
}

export interface DeleteUserRequest {
    uuid: string;
}

export interface GetUserRequest {
    uuid: string;
}

/**
 * 
 */
export class UserAdminManagementApi extends runtime.BaseAPI {

    /**
     * Create a user
     */
    async createUserRaw(requestParameters: CreateUserOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<UuidResponse>> {
        if (requestParameters.createUserRequest === null || requestParameters.createUserRequest === undefined) {
            throw new runtime.RequiredError('createUserRequest','Required parameter requestParameters.createUserRequest was null or undefined when calling createUser.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/admin/users`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: CreateUserRequestToJSON(requestParameters.createUserRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => UuidResponseFromJSON(jsonValue));
    }

    /**
     * Create a user
     */
    async createUser(requestParameters: CreateUserOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<UuidResponse> {
        const response = await this.createUserRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Delete a user by its uuid
     */
    async deleteUserRaw(requestParameters: DeleteUserRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling deleteUser.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/users/{uuid}`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'DELETE',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Delete a user by its uuid
     */
    async deleteUser(requestParameters: DeleteUserRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.deleteUserRaw(requestParameters, initOverrides);
    }

    /**
     * Retrieve all users
     */
    async getAllUsersAdminRaw(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<ListFullUsers>> {
        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/users`,
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => ListFullUsersFromJSON(jsonValue));
    }

    /**
     * Retrieve all users
     */
    async getAllUsersAdmin(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<ListFullUsers> {
        const response = await this.getAllUsersAdminRaw(initOverrides);
        return await response.value();
    }

    /**
     * Retrieve a user by its uuid
     */
    async getUserRaw(requestParameters: GetUserRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullUser>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling getUser.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/users/{uuid}`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullUserFromJSON(jsonValue));
    }

    /**
     * Retrieve a user by its uuid
     */
    async getUser(requestParameters: GetUserRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullUser> {
        const response = await this.getUserRaw(requestParameters, initOverrides);
        return await response.value();
    }

}
