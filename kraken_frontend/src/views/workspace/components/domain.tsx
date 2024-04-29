import { useCallback, useState } from "react";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import { DomainRelations, FullDomain, SimpleDomain } from "../../../api/generated";
import SelectableText from "../../../components/selectable-text";
import { handleApiError } from "../../../utils/helper";
import { DomainRelationsList } from "./relations-list";

/** React props for [`<Domain />`]{@link Domain} */
type DomainProps = {
    /**
     * Domain to display
     */
    domain: FullDomain | SimpleDomain;
    pretty?: boolean;
};

/**
 * Component to display domain.
 *
 * On hover display popup with list of domain relation
 */
export default function Domain(props: DomainProps) {
    const { domain } = props;
    const [relations, setRelations] = useState<DomainRelations | undefined>(undefined);

    const ensureDataLoaded = useCallback(() => {
        if (relations !== undefined) return;

        (async function () {
            const result = await Api.workspaces.domains.relations(domain.workspace, domain.uuid);
            handleApiError(result, (rels) => {
                setRelations(rels);
            });
        })();
    }, [domain.workspace, domain.uuid, relations, setRelations]);

    return (
        <Popup
            on={["hover", "focus"]}
            position={"right center"}
            arrow
            trigger={
                // eagerly load on mouse over, so popup potentially doesn't need to wait
                <div onMouseOver={ensureDataLoaded}>
                    <SelectableText>{domain.domain}</SelectableText>
                </div>
            }
            onOpen={ensureDataLoaded}
            keepTooltipInside
        >
            <DomainRelationsList
                className="workspace-data-details-relations-container pane-thin zero-padding-popup"
                relations={relations}
            />
        </Popup>
    );
}
