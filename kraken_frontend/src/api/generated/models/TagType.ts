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


/**
 * The type of a tag
 * @export
 */
export const TagType = {
    Workspace: 'Workspace',
    Global: 'Global'
} as const;
export type TagType = typeof TagType[keyof typeof TagType];


export function TagTypeFromJSON(json: any): TagType {
    return TagTypeFromJSONTyped(json, false);
}

export function TagTypeFromJSONTyped(json: any, ignoreDiscriminator: boolean): TagType {
    return json as TagType;
}

export function TagTypeToJSON(value?: TagType | null): any {
    return value as any;
}

