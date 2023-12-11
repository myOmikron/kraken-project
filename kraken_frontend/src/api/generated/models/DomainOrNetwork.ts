/**
 * @type DomainOrNetwork
 * Either an ip address / network or a domain name
 * @export
 */
export type DomainOrNetwork = string;

export function DomainOrNetworkFromJSON(json: any): DomainOrNetwork {
    return DomainOrNetworkFromJSONTyped(json, false);
}

export function DomainOrNetworkFromJSONTyped(json: any, ignoreDiscriminator: boolean): DomainOrNetwork {
        if (json === undefined || json === null || typeof json == "string") {
            return json;
        } else {
            throw TypeError("Invalid json");
        }
}

export function DomainOrNetworkToJSON(value?: DomainOrNetwork | null): any {
    return value;
}
