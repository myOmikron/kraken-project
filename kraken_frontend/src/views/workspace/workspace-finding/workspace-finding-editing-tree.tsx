import React, { useEffect, useRef } from "react";
import { ApiError, StatusCode } from "../../../api/error";
import {
    CreateFindingAffectedRequest,
    FindingSeverity,
    FullFinding,
    FullFindingAffected,
    SimpleFindingAffected,
    SimpleFindingDefinition,
} from "../../../api/generated";
import { Err, Ok, Result } from "../../../utils/result";
import { getAffectedData, getAffectedType } from "./workspace-edit-finding";
import {
    AffectedShallow,
    DynamicTreeGraph,
    DynamicTreeGraphProps,
    DynamicTreeGraphRef,
    DynamicTreeLookupFunctions,
    treeLookupFunctionsWorkspace,
} from "./workspace-finding-dynamic-tree";

export type EditingTreeGraphProps = {
    workspace: string;
    uuid?: string;
    definition?: SimpleFindingDefinition;
    severity: FindingSeverity;
    affected: (CreateFindingAffectedRequest | SimpleFindingAffected | FullFindingAffected)[];
} & Omit<DynamicTreeGraphProps, "workspace" | "uuid">;

export default function EditingTreeGraph(props: EditingTreeGraphProps) {
    const rootUuid = props.uuid ?? "local-finding";

    const treeRef = useRef<DynamicTreeGraphRef>(null);

    const getUuid = (affected: CreateFindingAffectedRequest | SimpleFindingAffected | FullFindingAffected) => {
        if ("affected" in affected) {
            return getAffectedData(affected).uuid;
        } else if ("affectedUuid" in affected) {
            return affected.affectedUuid;
        } else {
            return affected.uuid;
        }
    };

    const api = React.useRef<DynamicTreeLookupFunctions>({
        async getRoot() {
            throw new Error("function not overriden!");
        },
        async getAffected() {
            throw new Error("function not overriden!");
        },
        ...treeLookupFunctionsWorkspace(props.workspace),
    });
    // we update these functions so they don't use obsolete lambda context (e.g.
    // props.affected) every time, but we don't recreate the api / api.current
    // object so that the reference stays the same and the DynamicTreeGraph
    // component doesn't rerender because we changed how the API behaves every time.
    api.current.getRoot = async function (): Promise<FullFinding> {
        return {
            affected: props.affected.map((a, i) => ({
                _index: i,
                affectedUuid: getUuid(a),
                affectedType: "affected" in a ? getAffectedType(a) : "affectedType" in a ? a.affectedType : a.type,
                finding: rootUuid,
            })),
            createdAt: new Date(),
            definition: props.definition || {
                createdAt: new Date(),
                name: "(missing definition)",
                severity: props.severity,
                summary: "",
                uuid: "local-undefined",
            },
            userDetails: "",
            severity: props.severity,
            uuid: rootUuid,
        };
    };
    api.current.getAffected = async function ({
        affectedUuid: affected,
        _index,
    }: SimpleFindingAffected & { _index?: number }): Promise<Result<{ affected: AffectedShallow }, ApiError>> {
        if (_index === undefined)
            return Err({
                message: "invalid ID",
                status_code: StatusCode.ArbitraryJSError,
            });
        const obj = props.affected[_index];
        if ("affected" in obj) return Ok(obj);
        const uuid = getUuid(obj);
        let result: AffectedShallow;
        switch ("affectedType" in obj ? obj.affectedType : obj.type) {
            case "Domain":
                result = { domain: { uuid } };
                break;
            case "Host":
                result = { host: { uuid } };
                break;
            case "Port":
                result = { port: { uuid } };
                break;
            case "Service":
                result = { service: { uuid } };
                break;
        }
        return Ok({
            affected: result,
        });
    };

    useEffect(() => {
        treeRef.current?.reloadAffected();
    }, [props.affected]);

    return <DynamicTreeGraph ref={treeRef} uuid={rootUuid} api={api.current} {...props} />;
}
