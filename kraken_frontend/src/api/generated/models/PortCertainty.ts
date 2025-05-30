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
 * The certainty states of a port
 * @export
 */
export const PortCertainty = {
    Historical: 'Historical',
    SupposedTo: 'SupposedTo',
    Verified: 'Verified'
} as const;
export type PortCertainty = typeof PortCertainty[keyof typeof PortCertainty];


export function PortCertaintyFromJSON(json: any): PortCertainty {
    return PortCertaintyFromJSONTyped(json, false);
}

export function PortCertaintyFromJSONTyped(json: any, ignoreDiscriminator: boolean): PortCertainty {
    return json as PortCertainty;
}

export function PortCertaintyToJSON(value?: PortCertainty | null): any {
    return value as any;
}

