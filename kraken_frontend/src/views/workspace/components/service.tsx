import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullService, ServiceRelations, SimpleService } from "../../../api/generated";
import { handleApiError } from "../../../utils/helper";
import { ServiceRelationsList } from "./relations-list";

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
            <ServiceRelationsList
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}
