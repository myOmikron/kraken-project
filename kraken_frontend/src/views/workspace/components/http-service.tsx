import { useCallback, useEffect, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullHttpService, HttpServiceRelations, SimpleHttpService } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { buildHttpServiceURL } from "../../../utils/http-services";
import { HttpServiceRelationsList } from "./relations-list";

/**
 * Props for the <HttpServiceName> component.
 */
export type HttpServiceNameProps = {
    /**
     * The HTTP Service to show its name (and optionally URL if pretty is set)
     * of. If this is the simple variant and pretty is true, will load the full
     * variant from the API using this object's UUID.
     */
    httpService: FullHttpService | SimpleHttpService;
    /**
     * Can be set to true to show a full URL along the service name instead of
     * only the service name.
     */
    pretty?: boolean;
};

/**
 * Component showing a HTTP Service name and optionally related data.
 *
 * On hover this .shows relations in a popup.
 */
export default function HttpServiceName(props: HttpServiceNameProps) {
    const { httpService, pretty } = props;

    const [relations, setRelations] = useState<HttpServiceRelations | undefined>(undefined);
    const [fullHttpService, setFullHttpService] = useState<FullHttpService | undefined>(
        typeof httpService.host == "string" ? undefined : (httpService as FullHttpService),
    );

    useEffect(() => {
        if (pretty && (!fullHttpService || fullHttpService.uuid != httpService.uuid)) {
            setFullHttpService(undefined);
            Api.workspaces.httpServices
                .get(httpService.workspace, httpService.uuid)
                .then(handleApiError((s) => s.uuid == httpService.uuid && setFullHttpService(s)));
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
                    {pretty && fullHttpService ? (
                        <div>
                            <b>{httpService.name}</b>
                            {" on "}
                            <SelectableText as="span">{buildHttpServiceURL(fullHttpService, false)}</SelectableText>
                            {fullHttpService.domain && (
                                <>
                                    {" on "}
                                    <SelectableText as="span">{fullHttpService.host.ipAddr}</SelectableText>
                                </>
                            )}
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
