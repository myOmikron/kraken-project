import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { FullHost, HostRelations, SimpleHost } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { HostRelationsList } from "./relations-list";

/** React props for [`<IpAddr />`]{@link IpAddr} */
type IpAddrProps = {
    /**
     * Host to display
     */
    host: FullHost | SimpleHost;
    pretty?: boolean;
};

/**
 * Component to display a host/ ip address.
 *
 * On hover display popup with list of host relation
 */
export default function IpAddr(props: IpAddrProps) {
    const { host } = props;
    const [relations, setRelations] = useState<HostRelations | undefined>(undefined);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            const result = await Api.workspaces.hosts.relations(host.workspace, host.uuid);
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
