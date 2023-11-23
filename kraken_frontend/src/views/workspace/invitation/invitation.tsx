import React from "react";
import { FullWorkspace, SimpleUser, SimpleWorkspace } from "../../../api/generated";
import { Api, UUID } from "../../../api/api";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import "../../../styling/invitation.css";
import WorkspaceIcon from "../../../svg/workspace";

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

    componentDidUpdate(prevProps: Readonly<InvitationProps>, prevState: Readonly<InvitationState>, snapshot?: any) {
        if (prevProps !== this.props) {
            this.setState({ invitationPopup: true });
        }
    }

    async acceptInvitation() {
        (await Api.invitations.accept(this.props.invitationUuid)).match(
            () => this.props.onFinish(),
            (err) => {
                toast.error(err.message);
            }
        );
    }

    async declineInvitation() {
        (await Api.invitations.decline(this.props.invitationUuid)).match(
            () => this.props.onFinish(),
            (err) => {
                toast.error(err.message);
            }
        );
    }

    render() {
        console.log(this.props, this.state);
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
