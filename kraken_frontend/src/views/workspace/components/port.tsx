import { useCallback, useEffect, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullPort, PortRelations, SimplePort } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { PortRelationsList } from "./relations-list";

export default function PortNumber({
    port,
    pretty,
    withProtocol,
}: {
    port: FullPort | SimplePort;
    pretty?: boolean;
    withProtocol?: boolean;
}) {
    const [relations, setRelations] = useState<PortRelations | undefined>(undefined);
    const [fullPort, setFullPort] = useState<FullPort | undefined>(
        typeof port.host == "string" ? undefined : (port as FullPort),
    );

    useEffect(() => {
        if (pretty && (!fullPort || fullPort.uuid != port.uuid)) {
            setFullPort(undefined);
            Api.workspaces.ports
                .get(port.workspace, port.uuid)
                .then(handleApiError((s) => s.uuid == port.uuid && setFullPort(s)));
        }
    }, [port.uuid]);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            const result = await Api.workspaces.ports.relations(port.workspace, port.uuid);
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
                    {pretty && fullPort ? (
                        <div>
                            <b>
                                {port.protocol.toUpperCase()} {port.port}
                            </b>
                            {" on "}
                            <SelectableText as="span">{fullPort.host.ipAddr}</SelectableText>
                        </div>
                    ) : (
                        <div>
                            {port.port}
                            {withProtocol && `/${port.protocol.toUpperCase()}`}
                        </div>
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
