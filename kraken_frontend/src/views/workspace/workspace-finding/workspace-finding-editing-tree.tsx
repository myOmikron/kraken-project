import { ApiError, StatusCode } from "../../../api/error";
import {
    AggregationType,
    CreateFindingAffectedRequest,
    FindingSeverity,
    FullFinding,
    FullFindingAffected,
    SimpleFindingAffected,
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
    uuid?: string;
    definition?: SimpleFindingDefinition;
    severity: FindingSeverity;
    affected: (CreateFindingAffectedRequest | SimpleFindingAffected | FullFindingAffected)[];
} & Omit<DynamicTreeGraphProps, "workspace" | "uuid">;

export default function EditingTreeGraph(props: EditingTreeGraphProps) {
    const rootUuid = props.uuid ?? "local-finding";

    const affectedType = (a: FullFindingAffected): AggregationType =>
        "domain" in a.affected
            ? "Domain"
            : "host" in a.affected
              ? "Host"
              : "port" in a.affected
                ? "Port"
                : "service" in a.affected
                  ? "Service"
                  : (() => {
                        throw new Error("unexpected finding type");
                    })();

    const api: DynamicTreeLookupFunctions = {
        getRoot: async function (): Promise<FullFinding> {
            return {
                affected: props.affected.map((a, i) => ({
                    affectedUuid: "local-" + i,
                    affectedType: "affected" in a ? affectedType(a) : "affectedType" in a ? a.affectedType : a.type,
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
        },
        getAffected: async function (affected: string): Promise<Result<{ affected: AffectedShallow }, ApiError>> {
            if (!affected.startsWith("local-"))
                return Err({
                    message: "invalid ID",
                    status_code: StatusCode.ArbitraryJSError,
                });
            const obj = props.affected[parseInt(affected.substring("local-".length))];
            if ("affected" in obj) return Ok(obj);
            const uuid = "affectedUuid" in obj ? obj.affectedUuid : obj.uuid;
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
        },
        ...treeLookupFunctionsWorkspace(props.workspace),
    };
    return <DynamicTreeGraph uuid={rootUuid} api={api} {...props} />;
}
