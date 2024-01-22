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

import { exists, mapValues } from '../runtime';
import type { TestSSLSection } from './TestSSLSection';
import {
    TestSSLSectionFromJSON,
    TestSSLSectionFromJSONTyped,
    TestSSLSectionToJSON,
} from './TestSSLSection';
import type { TestSSLSeverity } from './TestSSLSeverity';
import {
    TestSSLSeverityFromJSON,
    TestSSLSeverityFromJSONTyped,
    TestSSLSeverityToJSON,
} from './TestSSLSeverity';

/**
 * A single finding reported by `testssl.sh`
 * 
 * This includes, log messages, extracted information (for example cert parameters) and tests for vulnerabilities / bad options.
 * @export
 * @interface TestSSLFinding
 */
export interface TestSSLFinding {
    /**
     * 
     * @type {TestSSLSection}
     * @memberof TestSSLFinding
     */
    section: TestSSLSection;
    /**
     * The finding's id (not db id, but `testssl.sh` id)
     * @type {string}
     * @memberof TestSSLFinding
     */
    id: string;
    /**
     * The finding's value (the value's semantics are highly dependant on the `id` and `severity`)
     * @type {string}
     * @memberof TestSSLFinding
     */
    value: string;
    /**
     * 
     * @type {TestSSLSeverity}
     * @memberof TestSSLFinding
     */
    severity: TestSSLSeverity;
    /**
     * An associated cve
     * @type {string}
     * @memberof TestSSLFinding
     */
    cve?: string | null;
    /**
     * An associated cwe category
     * @type {string}
     * @memberof TestSSLFinding
     */
    cwe?: string | null;
    /**
     * 
     * @type {any}
     * @memberof TestSSLFinding
     */
    issue: any | null;
}

/**
 * Check if a given object implements the TestSSLFinding interface.
 */
export function instanceOfTestSSLFinding(value: object): boolean {
    let isInstance = true;
    isInstance = isInstance && "section" in value;
    isInstance = isInstance && "id" in value;
    isInstance = isInstance && "value" in value;
    isInstance = isInstance && "severity" in value;
    isInstance = isInstance && "issue" in value;

    return isInstance;
}

export function TestSSLFindingFromJSON(json: any): TestSSLFinding {
    return TestSSLFindingFromJSONTyped(json, false);
}

export function TestSSLFindingFromJSONTyped(json: any, ignoreDiscriminator: boolean): TestSSLFinding {
    if ((json === undefined) || (json === null)) {
        return json;
    }
    return {
        
        'section': TestSSLSectionFromJSON(json['section']),
        'id': json['id'],
        'value': json['value'],
        'severity': TestSSLSeverityFromJSON(json['severity']),
        'cve': !exists(json, 'cve') ? undefined : json['cve'],
        'cwe': !exists(json, 'cwe') ? undefined : json['cwe'],
        'issue': json['issue'],
    };
}

export function TestSSLFindingToJSON(value?: TestSSLFinding | null): any {
    if (value === undefined) {
        return undefined;
    }
    if (value === null) {
        return null;
    }
    return {
        
        'section': TestSSLSectionToJSON(value.section),
        'id': value.id,
        'value': value.value,
        'severity': TestSSLSeverityToJSON(value.severity),
        'cve': value.cve,
        'cwe': value.cwe,
        'issue': value.issue,
    };
}

