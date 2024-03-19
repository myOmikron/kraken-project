import { ApiError, StatusCode } from "../../../api/error";
import {
    CreateFindingAffectedRequest,
    FindingSeverity,
    FullFinding,
    SimpleFindingDefinition,
} from "../../../api/generated";
import { Err, Ok, Result } from "../../../utils/result";
import {
    AffectedShallow,
    DynamicTreeGraph,
    DynamicTreeGraphProps,
    DynamicTreeLookupFunctions,
    treeLookupFunctionsWorkspace,
} from "./workspace-finding-dynamic-tree";

export type EditingTreeGraphProps = {
    workspace: string;
    definition?: SimpleFindingDefinition;
    severity: FindingSeverity;
    affected: CreateFindingAffectedRequest[];
} & Omit<DynamicTreeGraphProps, "workspace" | "uuid">;

export default function EditingTreeGraph(props: EditingTreeGraphProps) {
    const rootUuid = "editing:" + Math.random(); // TODO

    const api: DynamicTreeLookupFunctions = {
        getRoot: async function (): Promise<FullFinding> {
            return {
                affected: props.affected.map((a, i) => ({
                    affectedUuid: "local-" + i,
                    affectedType: a.type,
                    finding: "local-" + i,
                })),
                createdAt: new Date(),
                definition: props.definition || {
                    createdAt: new Date(),
                    name: "(missing definition)",
                    severity: props.severity,
                    summary: "",
                    uuid: "local-undefined",
                },
                severity: props.severity,
                uuid: rootUuid,
            };
        },
        getAffected: async function (affected: string): Promise<Result<{ affected: AffectedShallow }, ApiError>> {
            if (!affected.startsWith("local-"))
                return Err({
                    message: "invalid ID",
                    status_code: StatusCode.ArbitraryJSError,
                });
            const obj = props.affected[parseInt(affected.substring("local-".length))];
            let result: AffectedShallow;
            switch (obj.type) {
                case "Domain":
                    result = { domain: { uuid: obj.uuid } };
                    break;
                case "Host":
                    result = { host: { uuid: obj.uuid } };
                    break;
                case "Port":
                    result = { port: { uuid: obj.uuid } };
                    break;
                case "Service":
                    result = { service: { uuid: obj.uuid } };
                    break;
            }
            return Ok({
                affected: result,
            });
        },
        ...treeLookupFunctionsWorkspace(props.workspace),
    };
    return <DynamicTreeGraph uuid={rootUuid} api={api} {...props} />;
}
