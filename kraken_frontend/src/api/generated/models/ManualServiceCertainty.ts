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


/**
 * The certainty of a manually added service
 * @export
 */
export const ManualServiceCertainty = {
    Historical: 'Historical',
    SupposedTo: 'SupposedTo'
} as const;
export type ManualServiceCertainty = typeof ManualServiceCertainty[keyof typeof ManualServiceCertainty];


export function ManualServiceCertaintyFromJSON(json: any): ManualServiceCertainty {
    return ManualServiceCertaintyFromJSONTyped(json, false);
}

export function ManualServiceCertaintyFromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualServiceCertainty {
    return json as ManualServiceCertainty;
}

export function ManualServiceCertaintyToJSON(value?: ManualServiceCertainty | null): any {
    return value as any;
}
