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


/**
 * A representation of an OS type
 * @export
 */
export const OsType = {
    Unknown: 'Unknown',
    Linux: 'Linux',
    Windows: 'Windows',
    Apple: 'Apple',
    Android: 'Android',
    FreeBsd: 'FreeBSD'
} as const;
export type OsType = typeof OsType[keyof typeof OsType];


export function OsTypeFromJSON(json: any): OsType {
    return OsTypeFromJSONTyped(json, false);
}

export function OsTypeFromJSONTyped(json: any, ignoreDiscriminator: boolean): OsType {
    return json as OsType;
}

export function OsTypeToJSON(value?: OsType | null): any {
    return value as any;
}
