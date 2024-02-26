import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullService, ServiceRelations, SimpleService } from "../../../api/generated";
import RelationLeftIcon from "../../../svg/relation-left";
import { handleApiError } from "../../../utils/helper";

export default function ServiceName({ service }: { service: FullService | SimpleService }) {
    const [relations, setRelations] = useState<ServiceRelations | undefined>(undefined);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            let result = await Api.workspaces.services.relations(service.workspace, service.uuid);
            handleApiError(result, (rels) => {
                setRelations(rels);
            });
        })();
    }, [service.workspace, service.uuid, relations, setRelations]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded}>
                    <div>{service.name}</div>
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <ServiceRelationsView
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}

export function ServiceRelationsView({
    relations,
    ...props
}: { relations: ServiceRelations | null | undefined } & React.HTMLProps<HTMLDivElement>) {
    return (
        <div className="workspace-data-details-relations-container" {...props}>
            <div className="workspace-data-details-relations-header">
                <div className="workspace-data-details-relations-heading">Connection</div>
                <div className="workspace-data-details-relations-heading">Type</div>
                <div className="workspace-data-details-relations-heading">To</div>
            </div>
            {relations ? (
                <div className="workspace-data-details-relations-body">
                    {relations.host !== null && relations.host !== undefined ? (
                        <div className="workspace-data-details-relations-entry">
                            <div title={"Direct"}>
                                <RelationLeftIcon />
                            </div>
                            <span>Host</span>
                            <span>{relations.host.ipAddr} </span>
                        </div>
                    ) : undefined}
                    {relations.port !== null && relations.port !== undefined ? (
                        <div className="workspace-data-details-relations-entry">
                            <div title={"Direct"}>
                                <RelationLeftIcon />
                            </div>
                            <span>Port</span>
                            <span>{relations.port.port} </span>
                        </div>
                    ) : undefined}
                </div>
            ) : (
                <p>Loading...</p>
            )}
        </div>
    );
}
