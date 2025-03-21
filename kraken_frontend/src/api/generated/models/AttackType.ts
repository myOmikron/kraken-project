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
 * The type of attack
 * @export
 */
export const AttackType = {
    Undefined: 'Undefined',
    BruteforceSubdomains: 'BruteforceSubdomains',
    QueryCertificateTransparency: 'QueryCertificateTransparency',
    QueryUnhashed: 'QueryUnhashed',
    HostAlive: 'HostAlive',
    ServiceDetection: 'ServiceDetection',
    UdpServiceDetection: 'UdpServiceDetection',
    DnsResolution: 'DnsResolution',
    DnsTxtScan: 'DnsTxtScan',
    UdpPortScan: 'UdpPortScan',
    ForcedBrowsing: 'ForcedBrowsing',
    OsDetection: 'OSDetection',
    VersionDetection: 'VersionDetection',
    AntiPortScanningDetection: 'AntiPortScanningDetection',
    TestSsl: 'TestSSL'
} as const;
export type AttackType = typeof AttackType[keyof typeof AttackType];


export function AttackTypeFromJSON(json: any): AttackType {
    return AttackTypeFromJSONTyped(json, false);
}

export function AttackTypeFromJSONTyped(json: any, ignoreDiscriminator: boolean): AttackType {
    return json as AttackType;
}

export function AttackTypeToJSON(value?: AttackType | null): any {
    return value as any;
}

