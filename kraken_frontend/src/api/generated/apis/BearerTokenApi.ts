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
  CreateBearerTokenRequest,
  ListBearerTokens,
  UuidResponse,
} from '../models';
import {
    ApiErrorResponseFromJSON,
    ApiErrorResponseToJSON,
    CreateBearerTokenRequestFromJSON,
    CreateBearerTokenRequestToJSON,
    ListBearerTokensFromJSON,
    ListBearerTokensToJSON,
    UuidResponseFromJSON,
    UuidResponseToJSON,
} from '../models';

export interface CreateBearerTokenOperationRequest {
    createBearerTokenRequest: CreateBearerTokenRequest;
}

export interface DeleteBearerTokenRequest {
    uuid: string;
}

/**
 * 
 */
export class BearerTokenApi extends runtime.BaseAPI {

    /**
     * Create a new bearer token
     */
    async createBearerTokenRaw(requestParameters: CreateBearerTokenOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<UuidResponse>> {
        if (requestParameters.createBearerTokenRequest === null || requestParameters.createBearerTokenRequest === undefined) {
            throw new runtime.RequiredError('createBearerTokenRequest','Required parameter requestParameters.createBearerTokenRequest was null or undefined when calling createBearerToken.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/admin/bearer`,
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: CreateBearerTokenRequestToJSON(requestParameters.createBearerTokenRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => UuidResponseFromJSON(jsonValue));
    }

    /**
     * Create a new bearer token
     */
    async createBearerToken(requestParameters: CreateBearerTokenOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<UuidResponse> {
        const response = await this.createBearerTokenRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Delete an existing token
     */
    async deleteBearerTokenRaw(requestParameters: DeleteBearerTokenRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling deleteBearerToken.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/bearer/{uuid}`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'DELETE',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Delete an existing token
     */
    async deleteBearerToken(requestParameters: DeleteBearerTokenRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.deleteBearerTokenRaw(requestParameters, initOverrides);
    }

    /**
     * List all available bearer tokens
     */
    async listAllBearerTokensRaw(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<ListBearerTokens>> {
        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/bearer`,
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => ListBearerTokensFromJSON(jsonValue));
    }

    /**
     * List all available bearer tokens
     */
    async listAllBearerTokens(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<ListBearerTokens> {
        const response = await this.listAllBearerTokensRaw(initOverrides);
        return await response.value();
    }

}
