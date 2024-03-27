import React from "react";
import Popup from "reactjs-popup";
import { Api, UUID } from "../../../api/api";
import { SimpleUser, SimpleWorkspace } from "../../../api/generated";
import "../../../styling/invitation.css";
import WorkspaceIcon from "../../../svg/workspace";
import { handleApiError } from "../../../utils/helper";

type InvitationProps = {
    invitationUuid: UUID;
    workspace: SimpleWorkspace;
    from: SimpleUser;
    onFinish(): void;
};
type InvitationState = {};

export default class Invitation extends React.Component<InvitationProps, InvitationState> {
    constructor(props: InvitationProps) {
        super(props);

        this.state = {};
    }

    componentDidUpdate(prevProps: Readonly<InvitationProps>) {
        if (prevProps !== this.props) {
            this.setState({ invitationPopup: true });
        }
    }

    async acceptInvitation() {
        await Api.invitations.accept(this.props.invitationUuid).then(handleApiError(() => this.props.onFinish()));
    }

    async declineInvitation() {
        await Api.invitations.decline(this.props.invitationUuid).then(handleApiError(() => this.props.onFinish()));
    }

    render() {
        return (
            <Popup closeOnEscape={false} closeOnDocumentClick={false} modal={true} nested={true} open={true}>
                <div className="invitation-popup pane">
                    <div className="invitation-icon">
                        <WorkspaceIcon />
                    </div>
                    <h2 className="sub-heading">Workspace invitation</h2>
                    <span className="invitation-text">
                        <span>
                            {this.props.from.displayName} ({this.props.from.username}) invited you to join
                        </span>
                        <span>"{this.props.workspace.name}"</span>
                    </span>
                    <button
                        className="invitation-button  button"
                        onClick={async (e) => {
                            e.preventDefault();
                            await this.acceptInvitation();
                        }}
                    >
                        Accept
                    </button>
                    <button
                        className="invitation-button button"
                        onClick={async (e) => {
                            e.preventDefault();
                            await this.declineInvitation();
                        }}
                    >
                        Decline
                    </button>
                </div>
            </Popup>
        );
    }
}
