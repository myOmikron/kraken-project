import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullHost, HostRelations, SimpleHost } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { HostRelationsList } from "./relations-list";

export default function IpAddr({ host, pretty: _ }: { host: FullHost | SimpleHost; pretty?: boolean }) {
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
            <HostRelationsList
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}
