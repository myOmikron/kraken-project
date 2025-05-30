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
 * The certainty of a manually added http service
 * @export
 */
export const ManualHttpServiceCertainty = {
    Historical: 'Historical',
    SupposedTo: 'SupposedTo'
} as const;
export type ManualHttpServiceCertainty = typeof ManualHttpServiceCertainty[keyof typeof ManualHttpServiceCertainty];


export function ManualHttpServiceCertaintyFromJSON(json: any): ManualHttpServiceCertainty {
    return ManualHttpServiceCertaintyFromJSONTyped(json, false);
}

export function ManualHttpServiceCertaintyFromJSONTyped(json: any, ignoreDiscriminator: boolean): ManualHttpServiceCertainty {
    return json as ManualHttpServiceCertainty;
}

export function ManualHttpServiceCertaintyToJSON(value?: ManualHttpServiceCertainty | null): any {
    return value as any;
}

