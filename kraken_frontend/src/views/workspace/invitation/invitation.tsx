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

export default function Invitation(props: InvitationProps) {
    function acceptInvitation() {
        return Api.invitations.accept(props.invitationUuid).then(handleApiError(() => props.onFinish()));
    }

    function declineInvitation() {
        return Api.invitations.decline(props.invitationUuid).then(handleApiError(() => props.onFinish()));
    }

    return (
        <Popup closeOnEscape={false} closeOnDocumentClick={false} modal nested open>
            <div className="invitation-popup pane">
                <div className="invitation-icon">
                    <WorkspaceIcon />
                </div>
                <h2 className="sub-heading">Workspace invitation</h2>
                <span className="invitation-text">
                    <span>
                        {props.from.displayName} ({props.from.username}) invited you to join
                    </span>
                    <span>"{props.workspace.name}"</span>
                </span>
                <button
                    className="invitation-button button"
                    onClick={async (e) => {
                        e.preventDefault();
                        await acceptInvitation();
                    }}
                >
                    Accept
                </button>
                <button
                    className="invitation-button button"
                    onClick={async (e) => {
                        e.preventDefault();
                        await declineInvitation();
                    }}
                >
                    Decline
                </button>
            </div>
        </Popup>
    );
}
