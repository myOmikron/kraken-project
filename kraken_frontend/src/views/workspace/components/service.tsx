import { useCallback, useEffect, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullService, ServiceRelations, SimpleService } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { ServiceRelationsList } from "./relations-list";

/** React props for [`<ServiceName />`]{@link ServiceName} */
type ServiceNameProps = {
    /**
     * Service to display
     */
    service: FullService | SimpleService;
    pretty?: boolean;
};

/**
 * Component to display service name.
 *
 * On hover display popup with list of service relation
 */
export default function ServiceName(props: ServiceNameProps) {
    const { service, pretty } = props;
    const [relations, setRelations] = useState<ServiceRelations | undefined>(undefined);
    const [fullService, setFullService] = useState<FullService | undefined>(
        typeof service.host == "string" ? undefined : (service as FullService),
    );

    useEffect(() => {
        if (pretty && (!fullService || fullService.uuid != service.uuid)) {
            setFullService(undefined);
            Api.workspaces.services
                .get(service.workspace, service.uuid)
                .then(handleApiError((s) => s.uuid == service.uuid && setFullService(s)));
        }
    }, [service.uuid]);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            const result = await Api.workspaces.services.relations(service.workspace, service.uuid);
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
                    {pretty && fullService ? (
                        <div>
                            <b>{service.name}</b>
                            {" on "}
                            <SelectableText as="span">
                                {fullService.host.ipAddr.includes(":")
                                    ? `[${fullService.host.ipAddr}]`
                                    : fullService.host.ipAddr}
                                {typeof fullService.port === "object" && ":" + fullService.port?.port}
                            </SelectableText>
                        </div>
                    ) : (
                        <div>{service.name}</div>
                    )}
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
