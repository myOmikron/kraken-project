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
 * Protocols to select from when using `--starttls`
 * @export
 */
export const StartTLSProtocol = {
    Ftp: 'FTP',
    Smtp: 'SMTP',
    Pop3: 'POP3',
    Imap: 'IMAP',
    Xmpp: 'XMPP',
    Lmtp: 'LMTP',
    Nntp: 'NNTP',
    Postgres: 'Postgres',
    MySql: 'MySQL'
} as const;
export type StartTLSProtocol = typeof StartTLSProtocol[keyof typeof StartTLSProtocol];


export function StartTLSProtocolFromJSON(json: any): StartTLSProtocol {
    return StartTLSProtocolFromJSONTyped(json, false);
}

export function StartTLSProtocolFromJSONTyped(json: any, ignoreDiscriminator: boolean): StartTLSProtocol {
    return json as StartTLSProtocol;
}

export function StartTLSProtocolToJSON(value?: StartTLSProtocol | null): any {
    return value as any;
}

