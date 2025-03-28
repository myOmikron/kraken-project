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
  CreateFindingAffectedBulkRequest,
  CreateFindingAffectedRequest,
  CreateFindingRequest,
  FullFinding,
  FullFindingAffected,
  ListFindings,
  UpdateFindingAffectedRequest,
  UpdateFindingRequest,
  UuidResponse,
} from '../models';
import {
    ApiErrorResponseFromJSON,
    ApiErrorResponseToJSON,
    CreateFindingAffectedBulkRequestFromJSON,
    CreateFindingAffectedBulkRequestToJSON,
    CreateFindingAffectedRequestFromJSON,
    CreateFindingAffectedRequestToJSON,
    CreateFindingRequestFromJSON,
    CreateFindingRequestToJSON,
    FullFindingFromJSON,
    FullFindingToJSON,
    FullFindingAffectedFromJSON,
    FullFindingAffectedToJSON,
    ListFindingsFromJSON,
    ListFindingsToJSON,
    UpdateFindingAffectedRequestFromJSON,
    UpdateFindingAffectedRequestToJSON,
    UpdateFindingRequestFromJSON,
    UpdateFindingRequestToJSON,
    UuidResponseFromJSON,
    UuidResponseToJSON,
} from '../models';

export interface CreateFindingOperationRequest {
    uuid: string;
    createFindingRequest: CreateFindingRequest;
}

export interface CreateFindingAffectedOperationRequest {
    wUuid: string;
    fUuid: string;
    createFindingAffectedRequest: CreateFindingAffectedRequest;
}

export interface CreateFindingAffectedBulkOperationRequest {
    wUuid: string;
    fUuid: string;
    createFindingAffectedBulkRequest: CreateFindingAffectedBulkRequest;
}

export interface DeleteFindingRequest {
    wUuid: string;
    fUuid: string;
}

export interface DeleteFindingAffectedRequest {
    wUuid: string;
    fUuid: string;
    aUuid: string;
}

export interface GetAllFindingsRequest {
    uuid: string;
}

export interface GetFindingRequest {
    wUuid: string;
    fUuid: string;
}

export interface GetFindingAffectedRequest {
    wUuid: string;
    fUuid: string;
    aUuid: string;
}

export interface UpdateFindingOperationRequest {
    wUuid: string;
    fUuid: string;
    updateFindingRequest: UpdateFindingRequest;
}

export interface UpdateFindingAffectedOperationRequest {
    wUuid: string;
    fUuid: string;
    aUuid: string;
    updateFindingAffectedRequest: UpdateFindingAffectedRequest;
}

/**
 * 
 */
export class FindingsApi extends runtime.BaseAPI {

    /**
     * Creates a new finding
     */
    async createFindingRaw(requestParameters: CreateFindingOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<UuidResponse>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling createFinding.');
        }

        if (requestParameters.createFindingRequest === null || requestParameters.createFindingRequest === undefined) {
            throw new runtime.RequiredError('createFindingRequest','Required parameter requestParameters.createFindingRequest was null or undefined when calling createFinding.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspace/{uuid}/findings`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: CreateFindingRequestToJSON(requestParameters.createFindingRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => UuidResponseFromJSON(jsonValue));
    }

    /**
     * Creates a new finding
     */
    async createFinding(requestParameters: CreateFindingOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<UuidResponse> {
        const response = await this.createFindingRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Add a new affected object to a finding
     */
    async createFindingAffectedRaw(requestParameters: CreateFindingAffectedOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling createFindingAffected.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling createFindingAffected.');
        }

        if (requestParameters.createFindingAffectedRequest === null || requestParameters.createFindingAffectedRequest === undefined) {
            throw new runtime.RequiredError('createFindingAffectedRequest','Required parameter requestParameters.createFindingAffectedRequest was null or undefined when calling createFindingAffected.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}/affected`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))),
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: CreateFindingAffectedRequestToJSON(requestParameters.createFindingAffectedRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Add a new affected object to a finding
     */
    async createFindingAffected(requestParameters: CreateFindingAffectedOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.createFindingAffectedRaw(requestParameters, initOverrides);
    }

    /**
     * Add new affected objects in a bulk to a finding
     */
    async createFindingAffectedBulkRaw(requestParameters: CreateFindingAffectedBulkOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling createFindingAffectedBulk.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling createFindingAffectedBulk.');
        }

        if (requestParameters.createFindingAffectedBulkRequest === null || requestParameters.createFindingAffectedBulkRequest === undefined) {
            throw new runtime.RequiredError('createFindingAffectedBulkRequest','Required parameter requestParameters.createFindingAffectedBulkRequest was null or undefined when calling createFindingAffectedBulk.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}/affected-bulk`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))),
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: CreateFindingAffectedBulkRequestToJSON(requestParameters.createFindingAffectedBulkRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Add new affected objects in a bulk to a finding
     */
    async createFindingAffectedBulk(requestParameters: CreateFindingAffectedBulkOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.createFindingAffectedBulkRaw(requestParameters, initOverrides);
    }

    /**
     * Deletes a finding
     */
    async deleteFindingRaw(requestParameters: DeleteFindingRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling deleteFinding.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling deleteFinding.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))),
            method: 'DELETE',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Deletes a finding
     */
    async deleteFinding(requestParameters: DeleteFindingRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.deleteFindingRaw(requestParameters, initOverrides);
    }

    /**
     * Remove an affected object from a finding
     */
    async deleteFindingAffectedRaw(requestParameters: DeleteFindingAffectedRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling deleteFindingAffected.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling deleteFindingAffected.');
        }

        if (requestParameters.aUuid === null || requestParameters.aUuid === undefined) {
            throw new runtime.RequiredError('aUuid','Required parameter requestParameters.aUuid was null or undefined when calling deleteFindingAffected.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}/affected/{a_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))).replace(`{${"a_uuid"}}`, encodeURIComponent(String(requestParameters.aUuid))),
            method: 'DELETE',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Remove an affected object from a finding
     */
    async deleteFindingAffected(requestParameters: DeleteFindingAffectedRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.deleteFindingAffectedRaw(requestParameters, initOverrides);
    }

    /**
     * Gets a workspace\'s findings
     */
    async getAllFindingsRaw(requestParameters: GetAllFindingsRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<ListFindings>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling getAllFindings.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspace/{uuid}/findings`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => ListFindingsFromJSON(jsonValue));
    }

    /**
     * Gets a workspace\'s findings
     */
    async getAllFindings(requestParameters: GetAllFindingsRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<ListFindings> {
        const response = await this.getAllFindingsRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Gets a single finding
     */
    async getFindingRaw(requestParameters: GetFindingRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullFinding>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling getFinding.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling getFinding.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullFindingFromJSON(jsonValue));
    }

    /**
     * Gets a single finding
     */
    async getFinding(requestParameters: GetFindingRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullFinding> {
        const response = await this.getFindingRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Get an object affected by a finding
     */
    async getFindingAffectedRaw(requestParameters: GetFindingAffectedRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullFindingAffected>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling getFindingAffected.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling getFindingAffected.');
        }

        if (requestParameters.aUuid === null || requestParameters.aUuid === undefined) {
            throw new runtime.RequiredError('aUuid','Required parameter requestParameters.aUuid was null or undefined when calling getFindingAffected.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}/affected/{a_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))).replace(`{${"a_uuid"}}`, encodeURIComponent(String(requestParameters.aUuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullFindingAffectedFromJSON(jsonValue));
    }

    /**
     * Get an object affected by a finding
     */
    async getFindingAffected(requestParameters: GetFindingAffectedRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullFindingAffected> {
        const response = await this.getFindingAffectedRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Updates a finding
     */
    async updateFindingRaw(requestParameters: UpdateFindingOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling updateFinding.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling updateFinding.');
        }

        if (requestParameters.updateFindingRequest === null || requestParameters.updateFindingRequest === undefined) {
            throw new runtime.RequiredError('updateFindingRequest','Required parameter requestParameters.updateFindingRequest was null or undefined when calling updateFinding.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))),
            method: 'PUT',
            headers: headerParameters,
            query: queryParameters,
            body: UpdateFindingRequestToJSON(requestParameters.updateFindingRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Updates a finding
     */
    async updateFinding(requestParameters: UpdateFindingOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.updateFindingRaw(requestParameters, initOverrides);
    }

    /**
     * Update the details of an affected object
     */
    async updateFindingAffectedRaw(requestParameters: UpdateFindingAffectedOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling updateFindingAffected.');
        }

        if (requestParameters.fUuid === null || requestParameters.fUuid === undefined) {
            throw new runtime.RequiredError('fUuid','Required parameter requestParameters.fUuid was null or undefined when calling updateFindingAffected.');
        }

        if (requestParameters.aUuid === null || requestParameters.aUuid === undefined) {
            throw new runtime.RequiredError('aUuid','Required parameter requestParameters.aUuid was null or undefined when calling updateFindingAffected.');
        }

        if (requestParameters.updateFindingAffectedRequest === null || requestParameters.updateFindingAffectedRequest === undefined) {
            throw new runtime.RequiredError('updateFindingAffectedRequest','Required parameter requestParameters.updateFindingAffectedRequest was null or undefined when calling updateFindingAffected.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspace/{w_uuid}/findings/{f_uuid}/affected/{a_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"f_uuid"}}`, encodeURIComponent(String(requestParameters.fUuid))).replace(`{${"a_uuid"}}`, encodeURIComponent(String(requestParameters.aUuid))),
            method: 'PUT',
            headers: headerParameters,
            query: queryParameters,
            body: UpdateFindingAffectedRequestToJSON(requestParameters.updateFindingAffectedRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Update the details of an affected object
     */
    async updateFindingAffected(requestParameters: UpdateFindingAffectedOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.updateFindingAffectedRaw(requestParameters, initOverrides);
    }

}
