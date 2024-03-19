import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullPort, PortRelations, SimplePort } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { PortRelationsList } from "./relations-list";

export default function PortNumber({ port, pretty }: { port: FullPort | SimplePort; pretty?: boolean }) {
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
                    {pretty && typeof port.host == "object" ? (
                        <div>
                            <b>
                                {port.protocol.toUpperCase()} {port.port}
                            </b>
                            {" on "}
                            <SelectableText as="span">{port.host.ipAddr}</SelectableText>
                        </div>
                    ) : (
                        <div>{port.port}</div>
                    )}
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <PortRelationsList
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}
