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

type GlobalPopupProps = {};
type GlobalPopupState = {
    popups: Array<Popup>;
};

export default class GlobalPopup extends React.Component<GlobalPopupProps, GlobalPopupState> {
    constructor(props: GlobalPopupProps) {
        super(props);

        this.state = {
            popups: [],
        };
    }

    componentDidMount() {
        this.retrieveInvitations().then();
        WS.addEventListener("message.InvitationToWorkspace", (e) => {
            toast.info("Invitation received");
            const popups = this.state.popups;
            popups.push({ type: "invitation", invitationUuid: e.invitationUuid, from: e.from, workspace: e.workspace });
            this.setState({ popups });
        });
    }

    async retrieveInvitations() {
        await Api.invitations.all().then(
            handleApiError((e) => {
                const popups = this.state.popups;
                popups.push(
                    ...e.invitations.map((i): WsInvitationToWorkspace => {
                        return { type: "invitation", invitationUuid: i.uuid, from: i.from, workspace: i.workspace };
                    }),
                );
                this.setState({ popups });
            }),
        );
    }

    render() {
        console.log(
            this.state.popups.map((e) => {
                return e.workspace.name;
            }),
        );
        if (this.state.popups.length !== 0) {
            const popup = this.state.popups[0];
            switch (popup.type) {
                case "invitation":
                    return (
                        <Invitation
                            workspace={popup.workspace}
                            invitationUuid={popup.invitationUuid}
                            from={popup.from}
                            onFinish={() => {
                                const popups = this.state.popups;
                                popups.splice(0, 1);
                                this.setState({ popups });
                            }}
                        />
                    );
            }
        }
    }
}
