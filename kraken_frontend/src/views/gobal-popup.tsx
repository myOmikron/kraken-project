import React from "react";
import { toast } from "react-toastify";
import { Api, UUID } from "../api/api";
import { SimpleUser, SimpleWorkspace } from "../api/generated";
import WS from "../api/websocket";
import { handleApiError } from "../utils/helper";
import Invitation from "./workspace/invitation/invitation";

type Popup = WsInvitationToWorkspace;

type WsInvitationToWorkspace = {
    type: "invitation";
    invitationUuid: UUID;
    workspace: SimpleWorkspace;
    from: SimpleUser;
};

export default function GlobalPopup() {
    const [popups, setPopups] = React.useState<Array<Popup>>([]);

    /** api call to retrieve all invitations and push them into popup list */
    async function retrieveInvitations() {
        await Api.invitations.all().then(
            handleApiError((e) => {
                popups.push(
                    ...e.invitations.map((i): WsInvitationToWorkspace => {
                        return { type: "invitation", invitationUuid: i.uuid, from: i.from, workspace: i.workspace };
                    }),
                );
                setPopups(popups);
            }),
        );
    }

    React.useEffect(() => {
        retrieveInvitations().then();
        WS.addEventListener("message.InvitationToWorkspace", (e) => {
            toast.info("Invitation received");
            popups.push({ type: "invitation", invitationUuid: e.invitationUuid, from: e.from, workspace: e.workspace });
            setPopups(popups);
        });
    }, []);

    let popup: Popup;
    if (popups.length !== 0) {
        popup = popups[0];
    }

    const popupDisplay = () => {
        switch (popup.type) {
            case "invitation":
                return (
                    <Invitation
                        workspace={popup.workspace}
                        invitationUuid={popup.invitationUuid}
                        from={popup.from}
                        onFinish={() => {
                            const p = popups;
                            p.splice(0, 1);
                            setPopups(p);
                        }}
                    />
                );
        }
    };

    return <>{popupDisplay}</>;
}
