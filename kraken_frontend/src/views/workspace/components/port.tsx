import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullPort, PortRelations, SimplePort } from "../../../api/generated";
import RelationLeftIcon from "../../../svg/relation-left";
import RelationRightIcon from "../../../svg/relation-right";
import { handleApiError } from "../../../utils/helper";

export default function PortNumber({ port }: { port: FullPort | SimplePort }) {
    const [relations, setRelations] = useState<PortRelations | undefined>(undefined);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            let result = await Api.workspaces.ports.relations(port.workspace, port.uuid);
            handleApiError(result, (rels) => {
                setRelations(rels);
            });
        })();
    }, [port.workspace, port.uuid, relations, setRelations]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded}>
                    <div>{port.port}</div>
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <PortRelationsView
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}

export function PortRelationsView({
    relations,
    ...props
}: { relations: PortRelations | null | undefined } & React.HTMLProps<HTMLDivElement>) {
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
