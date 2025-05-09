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
import type { DomainCertainty } from './DomainCertainty';
import {
    DomainCertaintyFromJSON,
    DomainCertaintyFromJSONTyped,
    DomainCertaintyToJSON,
} from './DomainCertainty';

/**
 * A simple representation of a domain in a workspace
 * @export
 * @interface SimpleDomain
 */
export interface SimpleDomain {
    /**
     * The uuid of the domain
     * @type {string}
     * @memberof SimpleDomain
     */
    uuid: string;
    /**
     * The domain name
     * @type {string}
     * @memberof SimpleDomain
     */
    domain: string;
    /**
     * The comment to the domain
     * @type {string}
     * @memberof SimpleDomain
     */
    comment: string;
    /**
     * The workspace this domain is linked to
     * @type {string}
     * @memberof SimpleDomain
     */
    workspace: string;
    /**
     * The point in time this domain was created
     * @type {Date}
     * @memberof SimpleDomain
     */
    createdAt: Date;
    /**
     * 
     * @type {DomainCertainty}
     * @memberof SimpleDomain
     */
    certainty: DomainCertainty;
}

/**
 * Check if a given object implements the SimpleDomain interface.
 */
export function instanceOfSimpleDomain(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "domain" in value;
    isInstance = isInstance && "comment" in value;
    isInstance = isInstance && "workspace" in value;
    isInstance = isInstance && "createdAt" in value;
    isInstance = isInstance && "certainty" in value;

    return isInstance;
}

export function SimpleDomainFromJSON(json: any): SimpleDomain {
    return SimpleDomainFromJSONTyped(json, false);
}

export function SimpleDomainFromJSONTyped(json: any, ignoreDiscriminator: boolean): SimpleDomain {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'domain': json['domain'],
        'comment': json['comment'],
        'workspace': json['workspace'],
        'createdAt': (new Date(json['created_at'])),
        'certainty': DomainCertaintyFromJSON(json['certainty']),
    };
}

export function SimpleDomainToJSON(value?: SimpleDomain | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'domain': value.domain,
        'comment': value.comment,
        'workspace': value.workspace,
        'created_at': (value.createdAt.toISOString()),
        'certainty': DomainCertaintyToJSON(value.certainty),
    };
}

