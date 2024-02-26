import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullHost, HostRelations, SimpleHost } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import RelationIndirectIcon from "../../../svg/relation-indirect";
import RelationLeftIcon from "../../../svg/relation-left";
import RelationRightIcon from "../../../svg/relation-right";
import { handleApiError } from "../../../utils/helper";

export default function IpAddr({ host }: { host: FullHost | SimpleHost }) {
    const [relations, setRelations] = useState<HostRelations | undefined>(undefined);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            let result = await Api.workspaces.hosts.relations(host.workspace, host.uuid);
            handleApiError(result, (rels) => {
                setRelations(rels);
            });
        })();
    }, [host.workspace, host.uuid, relations, setRelations]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded}>
                    <SelectableText>{host.ipAddr}</SelectableText>
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <HostRelationsView
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}

export function HostRelationsView({
    relations,
    ...props
}: { relations: HostRelations | null | undefined } & React.HTMLProps<HTMLDivElement>) {
    return (
        <div className="workspace-data-details-relations-container" {...props}>
            <div className="workspace-data-details-relations-header">
                <div className="workspace-data-details-relations-heading">Connection</div>
                <div className="workspace-data-details-relations-heading">Type</div>
                <div className="workspace-data-details-relations-heading">To</div>
            </div>
            {relations ? (
                <div className="workspace-data-details-relations-body">
                    {relations.directDomains.map((d) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div title={"Direct"}>
                                    <RelationLeftIcon />
                                </div>
                                <span>Domain</span>
                                <span>{d.domain} </span>
                            </div>
                        );
                    })}
                    {relations.indirectDomains.map((d) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div className="indirect" title={"Indirect"}>
                                    <RelationIndirectIcon />
                                </div>
                                <span>Domain</span>
                                <span>{d.domain} </span>
                            </div>
                        );
                    })}
                    {relations.ports.map((p) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div title={"Direct"}>
                                    <RelationRightIcon />
                                </div>
                                <span>Port</span>
                                <span>{p.port} </span>
                            </div>
                        );
                    })}
                    {relations.services.map((s) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div title={"Direct"}>
                                    <RelationRightIcon />
                                </div>
                                <span>Service</span>
                                <span>{s.name} </span>
                            </div>
                        );
                    })}
                </div>
            ) : (
                <p>Loading...</p>
            )}
        </div>
    );
}
