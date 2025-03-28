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
 * The certainty a service is detected
 * @export
 */
export const ServiceCertainty = {
    Historical: 'Historical',
    SupposedTo: 'SupposedTo',
    MaybeVerified: 'MaybeVerified',
    DefinitelyVerified: 'DefinitelyVerified',
    UnknownService: 'UnknownService'
} as const;
export type ServiceCertainty = typeof ServiceCertainty[keyof typeof ServiceCertainty];


export function ServiceCertaintyFromJSON(json: any): ServiceCertainty {
    return ServiceCertaintyFromJSONTyped(json, false);
}

export function ServiceCertaintyFromJSONTyped(json: any, ignoreDiscriminator: boolean): ServiceCertainty {
    return json as ServiceCertainty;
}

export function ServiceCertaintyToJSON(value?: ServiceCertainty | null): any {
    return value as any;
}

