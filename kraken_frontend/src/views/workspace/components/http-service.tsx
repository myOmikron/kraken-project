import { useCallback, useEffect, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullHttpService } from "../../../api/generated/models/FullHttpService";
import { HttpServiceRelations } from "../../../api/generated/models/HttpServiceRelations";
import { SimpleHttpService } from "../../../api/generated/models/SimpleHttpService";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { HttpServiceRelationsList } from "./relations-list";

export default function HttpServiceName({
    httpService,
    pretty,
}: {
    httpService: FullHttpService | SimpleHttpService;
    pretty?: boolean;
}) {
    const [relations, setRelations] = useState<HttpServiceRelations | undefined>(undefined);
    const [fullService, setFullService] = useState<FullHttpService | undefined>(
        typeof httpService.host == "string" ? undefined : (httpService as FullHttpService),
    );

    useEffect(() => {
        if (pretty && (!fullService || fullService.uuid != httpService.uuid)) {
            setFullService(undefined);
            Api.workspaces.httpServices
                .get(httpService.workspace, httpService.uuid)
                .then(handleApiError((s) => s.uuid == httpService.uuid && setFullService(s)));
        }
    }, [httpService.uuid]);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            const result = await Api.workspaces.httpServices.relations(httpService.workspace, httpService.uuid);
            handleApiError(result, (rels) => {
                setRelations(rels);
            });
        })();
    }, [httpService.workspace, httpService.uuid, relations, setRelations]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded}>
                    {pretty && fullService ? (
                        <div>
                            <b>{httpService.name}</b>
                            {" on "}
                            <SelectableText as="span">
                                {fullService.host.ipAddr.includes(":")
                                    ? `[${fullService.host.ipAddr}]`
                                    : fullService.host.ipAddr}
                                {typeof fullService.port === "object" && ":" + fullService.port?.port}
                            </SelectableText>
                        </div>
                    ) : (
                        <div>{httpService.name}</div>
                    )}
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <HttpServiceRelationsList
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}
