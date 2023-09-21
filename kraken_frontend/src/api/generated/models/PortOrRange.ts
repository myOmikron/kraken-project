export type PortOrRange = number | string;

export function PortOrRangeFromJSON(json: any): PortOrRange {
    return PortOrRangeFromJSONTyped(json, false);
}

export function PortOrRangeFromJSONTyped(json: any, ignoreDiscriminator: boolean): PortOrRange {
    if (json === undefined || json === null || typeof json == "string" || typeof json == "number") {
        return json;
    } else {
        throw TypeError("Invalid json");
    }
}

export function PortOrRangeToJSON(value?: PortOrRange | null): any {
    return value;
}
