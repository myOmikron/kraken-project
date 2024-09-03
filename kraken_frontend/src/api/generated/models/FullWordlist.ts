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
 * A wordlist including its `path` field only meant for admins
 * @export
 * @interface FullWordlist
 */
export interface FullWordlist {
    /**
     * The primary key of the wordlist
     * @type {string}
     * @memberof FullWordlist
     */
    uuid: string;
    /**
     * The wordlist's name to be displayed select buttons
     * @type {string}
     * @memberof FullWordlist
     */
    name: string;
    /**
     * A description explaining the wordlist's intended use case
     * @type {string}
     * @memberof FullWordlist
     */
    description: string;
    /**
     * The file path the wordlist is deployed under on each leech
     * @type {string}
     * @memberof FullWordlist
     */
    path: string;
}

/**
 * Check if a given object implements the FullWordlist interface.
 */
export function instanceOfFullWordlist(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "uuid" in value;
    isInstance = isInstance && "name" in value;
    isInstance = isInstance && "description" in value;
    isInstance = isInstance && "path" in value;

    return isInstance;
}

export function FullWordlistFromJSON(json: any): FullWordlist {
    return FullWordlistFromJSONTyped(json, false);
}

export function FullWordlistFromJSONTyped(json: any, ignoreDiscriminator: boolean): FullWordlist {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'uuid': json['uuid'],
        'name': json['name'],
        'description': json['description'],
        'path': json['path'],
    };
}

export function FullWordlistToJSON(value?: FullWordlist | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'uuid': value.uuid,
        'name': value.name,
        'description': value.description,
        'path': value.path,
    };
}

