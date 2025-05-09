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
  FullWorkspace,
  ListWorkspaces,
} from '../models';
import {
    ApiErrorResponseFromJSON,
    ApiErrorResponseToJSON,
    FullWorkspaceFromJSON,
    FullWorkspaceToJSON,
    ListWorkspacesFromJSON,
    ListWorkspacesToJSON,
} from '../models';

export interface GetWorkspaceAdminRequest {
    uuid: string;
}

/**
 * 
 */
export class AdminWorkspacesApi extends runtime.BaseAPI {

    /**
     * Retrieve all workspaces
     */
    async getAllWorkspacesAdminRaw(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<ListWorkspaces>> {
        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/workspaces`,
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => ListWorkspacesFromJSON(jsonValue));
    }

    /**
     * Retrieve all workspaces
     */
    async getAllWorkspacesAdmin(initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<ListWorkspaces> {
        const response = await this.getAllWorkspacesAdminRaw(initOverrides);
        return await response.value();
    }

    /**
     * Retrieve a workspace by id
     */
    async getWorkspaceAdminRaw(requestParameters: GetWorkspaceAdminRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullWorkspace>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling getWorkspaceAdmin.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/admin/workspaces/{uuid}`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullWorkspaceFromJSON(jsonValue));
    }

    /**
     * Retrieve a workspace by id
     */
    async getWorkspaceAdmin(requestParameters: GetWorkspaceAdminRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullWorkspace> {
        const response = await this.getWorkspaceAdminRaw(requestParameters, initOverrides);
        return await response.value();
    }

}
