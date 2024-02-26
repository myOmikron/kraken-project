import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { DomainRelations, FullDomain, SimpleDomain } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import RelationIndirectIcon from "../../../svg/relation-indirect";
import RelationLeftIcon from "../../../svg/relation-left";
import RelationRightIcon from "../../../svg/relation-right";
import { handleApiError } from "../../../utils/helper";

export default function Domain({ domain }: { domain: FullDomain | SimpleDomain }) {
    const [relations, setRelations] = useState<DomainRelations | undefined>(undefined);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            let result = await Api.workspaces.domains.relations(domain.workspace, domain.uuid);
            handleApiError(result, (rels) => {
                setRelations(rels);
            });
        })();
    }, [domain.workspace, domain.uuid, relations, setRelations]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded}>
                    <SelectableText>{domain.domain}</SelectableText>
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <DomainRelationsView
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}

export function DomainRelationsView({
    relations,
    ...props
}: { relations: DomainRelations | null | undefined } & React.HTMLProps<HTMLDivElement>) {
    return (
        <div className="workspace-data-details-relations-container" {...props}>
            <div className="workspace-data-details-relations-header">
                <div className="workspace-data-details-relations-heading">Connection</div>
                <div className="workspace-data-details-relations-heading">Type</div>
                <div className="workspace-data-details-relations-heading">To</div>
            </div>
            {relations ? (
                <div className="workspace-data-details-relations-body">
                    {relations.sourceDomains.map((d) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div title={"Direct source"}>
                                    <RelationLeftIcon />
                                </div>
                                <span>Domain</span>
                                <span>{d.domain} </span>
                            </div>
                        );
                    })}
                    {relations.targetDomains.map((d) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div title={"Direct target"}>
                                    <RelationRightIcon />
                                </div>
                                <span>Domain</span>
                                <span>{d.domain} </span>
                            </div>
                        );
                    })}
                    {relations.directHosts.map((h) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div title={"Direct"}>
                                    <RelationRightIcon />
                                </div>
                                <span>Host</span>
                                <span>{h.ipAddr} </span>
                            </div>
                        );
                    })}
                    {relations.indirectHosts.map((h) => {
                        return (
                            <div className="workspace-data-details-relations-entry">
                                <div className="indirect" title={"Indirect"}>
                                    <RelationIndirectIcon />
                                </div>
                                <span>Host</span>
                                <span>{h.ipAddr} </span>
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
