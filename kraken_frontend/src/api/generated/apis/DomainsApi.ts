/* tslint:disable */
/* eslint-disable */
/**
 * kraken
 * The core component of kraken-project
 *
 * The version of the OpenAPI document: 0.1.0
 * Contact: git@omikron.dev
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */


import * as runtime from '../runtime';
import type {
  ApiErrorResponse,
  CreateDomainRequest,
  DomainRelations,
  DomainResultsPage,
  FullAggregationSource,
  FullDomain,
  GetAllDomainsQuery,
  UpdateDomainRequest,
  UuidResponse,
} from '../models';
import {
    ApiErrorResponseFromJSON,
    ApiErrorResponseToJSON,
    CreateDomainRequestFromJSON,
    CreateDomainRequestToJSON,
    DomainRelationsFromJSON,
    DomainRelationsToJSON,
    DomainResultsPageFromJSON,
    DomainResultsPageToJSON,
    FullAggregationSourceFromJSON,
    FullAggregationSourceToJSON,
    FullDomainFromJSON,
    FullDomainToJSON,
    GetAllDomainsQueryFromJSON,
    GetAllDomainsQueryToJSON,
    UpdateDomainRequestFromJSON,
    UpdateDomainRequestToJSON,
    UuidResponseFromJSON,
    UuidResponseToJSON,
} from '../models';

export interface CreateDomainOperationRequest {
    uuid: string;
    createDomainRequest: CreateDomainRequest;
}

export interface DeleteDomainRequest {
    wUuid: string;
    dUuid: string;
}

export interface GetAllDomainsRequest {
    uuid: string;
    getAllDomainsQuery: GetAllDomainsQuery;
}

export interface GetDomainRequest {
    wUuid: string;
    dUuid: string;
}

export interface GetDomainRelationsRequest {
    wUuid: string;
    dUuid: string;
}

export interface GetDomainSourcesRequest {
    wUuid: string;
    dUuid: string;
}

export interface UpdateDomainOperationRequest {
    wUuid: string;
    dUuid: string;
    updateDomainRequest: UpdateDomainRequest;
}

/**
 * 
 */
export class DomainsApi extends runtime.BaseAPI {

    /**
     * Manually add a domain
     * Manually add a domain
     */
    async createDomainRaw(requestParameters: CreateDomainOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<UuidResponse>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling createDomain.');
        }

        if (requestParameters.createDomainRequest === null || requestParameters.createDomainRequest === undefined) {
            throw new runtime.RequiredError('createDomainRequest','Required parameter requestParameters.createDomainRequest was null or undefined when calling createDomain.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspaces/{uuid}/domains`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: CreateDomainRequestToJSON(requestParameters.createDomainRequest),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => UuidResponseFromJSON(jsonValue));
    }

    /**
     * Manually add a domain
     * Manually add a domain
     */
    async createDomain(requestParameters: CreateDomainOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<UuidResponse> {
        const response = await this.createDomainRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Delete the domain  This only deletes the aggregation. The raw results are still in place
     * Delete the domain
     */
    async deleteDomainRaw(requestParameters: DeleteDomainRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling deleteDomain.');
        }

        if (requestParameters.dUuid === null || requestParameters.dUuid === undefined) {
            throw new runtime.RequiredError('dUuid','Required parameter requestParameters.dUuid was null or undefined when calling deleteDomain.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspaces/{w_uuid}/domains/{d_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"d_uuid"}}`, encodeURIComponent(String(requestParameters.dUuid))),
            method: 'DELETE',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Delete the domain  This only deletes the aggregation. The raw results are still in place
     * Delete the domain
     */
    async deleteDomain(requestParameters: DeleteDomainRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.deleteDomainRaw(requestParameters, initOverrides);
    }

    /**
     * Retrieve all domains of a specific workspace
     * Retrieve all domains of a specific workspace
     */
    async getAllDomainsRaw(requestParameters: GetAllDomainsRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<DomainResultsPage>> {
        if (requestParameters.uuid === null || requestParameters.uuid === undefined) {
            throw new runtime.RequiredError('uuid','Required parameter requestParameters.uuid was null or undefined when calling getAllDomains.');
        }

        if (requestParameters.getAllDomainsQuery === null || requestParameters.getAllDomainsQuery === undefined) {
            throw new runtime.RequiredError('getAllDomainsQuery','Required parameter requestParameters.getAllDomainsQuery was null or undefined when calling getAllDomains.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspaces/{uuid}/domains/all`.replace(`{${"uuid"}}`, encodeURIComponent(String(requestParameters.uuid))),
            method: 'POST',
            headers: headerParameters,
            query: queryParameters,
            body: GetAllDomainsQueryToJSON(requestParameters.getAllDomainsQuery),
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => DomainResultsPageFromJSON(jsonValue));
    }

    /**
     * Retrieve all domains of a specific workspace
     * Retrieve all domains of a specific workspace
     */
    async getAllDomains(requestParameters: GetAllDomainsRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<DomainResultsPage> {
        const response = await this.getAllDomainsRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Retrieve all information about a single domain
     * Retrieve all information about a single domain
     */
    async getDomainRaw(requestParameters: GetDomainRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullDomain>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling getDomain.');
        }

        if (requestParameters.dUuid === null || requestParameters.dUuid === undefined) {
            throw new runtime.RequiredError('dUuid','Required parameter requestParameters.dUuid was null or undefined when calling getDomain.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspaces/{w_uuid}/domains/{d_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"d_uuid"}}`, encodeURIComponent(String(requestParameters.dUuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullDomainFromJSON(jsonValue));
    }

    /**
     * Retrieve all information about a single domain
     * Retrieve all information about a single domain
     */
    async getDomain(requestParameters: GetDomainRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullDomain> {
        const response = await this.getDomainRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Get a host\'s direct relations
     * Get a host\'s direct relations
     */
    async getDomainRelationsRaw(requestParameters: GetDomainRelationsRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<DomainRelations>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling getDomainRelations.');
        }

        if (requestParameters.dUuid === null || requestParameters.dUuid === undefined) {
            throw new runtime.RequiredError('dUuid','Required parameter requestParameters.dUuid was null or undefined when calling getDomainRelations.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspaces/{w_uuid}/domains/{d_uuid}/relations`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"d_uuid"}}`, encodeURIComponent(String(requestParameters.dUuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => DomainRelationsFromJSON(jsonValue));
    }

    /**
     * Get a host\'s direct relations
     * Get a host\'s direct relations
     */
    async getDomainRelations(requestParameters: GetDomainRelationsRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<DomainRelations> {
        const response = await this.getDomainRelationsRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Get all data sources which referenced this domain
     * Get all data sources which referenced this domain
     */
    async getDomainSourcesRaw(requestParameters: GetDomainSourcesRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<FullAggregationSource>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling getDomainSources.');
        }

        if (requestParameters.dUuid === null || requestParameters.dUuid === undefined) {
            throw new runtime.RequiredError('dUuid','Required parameter requestParameters.dUuid was null or undefined when calling getDomainSources.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        const response = await this.request({
            path: `/api/v1/workspaces/{w_uuid}/domains/{d_uuid}/sources`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"d_uuid"}}`, encodeURIComponent(String(requestParameters.dUuid))),
            method: 'GET',
            headers: headerParameters,
            query: queryParameters,
        }, initOverrides);

        return new runtime.JSONApiResponse(response, (jsonValue) => FullAggregationSourceFromJSON(jsonValue));
    }

    /**
     * Get all data sources which referenced this domain
     * Get all data sources which referenced this domain
     */
    async getDomainSources(requestParameters: GetDomainSourcesRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<FullAggregationSource> {
        const response = await this.getDomainSourcesRaw(requestParameters, initOverrides);
        return await response.value();
    }

    /**
     * Update a domain  You must include at least on parameter
     * Update a domain
     */
    async updateDomainRaw(requestParameters: UpdateDomainOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<runtime.ApiResponse<void>> {
        if (requestParameters.wUuid === null || requestParameters.wUuid === undefined) {
            throw new runtime.RequiredError('wUuid','Required parameter requestParameters.wUuid was null or undefined when calling updateDomain.');
        }

        if (requestParameters.dUuid === null || requestParameters.dUuid === undefined) {
            throw new runtime.RequiredError('dUuid','Required parameter requestParameters.dUuid was null or undefined when calling updateDomain.');
        }

        if (requestParameters.updateDomainRequest === null || requestParameters.updateDomainRequest === undefined) {
            throw new runtime.RequiredError('updateDomainRequest','Required parameter requestParameters.updateDomainRequest was null or undefined when calling updateDomain.');
        }

        const queryParameters: any = {};

        const headerParameters: runtime.HTTPHeaders = {};

        headerParameters['Content-Type'] = 'application/json';

        const response = await this.request({
            path: `/api/v1/workspaces/{w_uuid}/domains/{d_uuid}`.replace(`{${"w_uuid"}}`, encodeURIComponent(String(requestParameters.wUuid))).replace(`{${"d_uuid"}}`, encodeURIComponent(String(requestParameters.dUuid))),
            method: 'PUT',
            headers: headerParameters,
            query: queryParameters,
            body: UpdateDomainRequestToJSON(requestParameters.updateDomainRequest),
        }, initOverrides);

        return new runtime.VoidApiResponse(response);
    }

    /**
     * Update a domain  You must include at least on parameter
     * Update a domain
     */
    async updateDomain(requestParameters: UpdateDomainOperationRequest, initOverrides?: RequestInit | runtime.InitOverrideFunction): Promise<void> {
        await this.updateDomainRaw(requestParameters, initOverrides);
    }

}
