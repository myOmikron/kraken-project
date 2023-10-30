import React from "react";
import { Api } from "../../api/api";
import { toast } from "react-toastify";
import { GetUser, UserPermission } from "../../api/generated/models";
import Loading from "../../components/loading";
import Popup from "reactjs-popup";
import Input from "../../components/input";
import { check, handleApiError } from "../../utils/helper";

type AdminUsersProps = {};
type AdminUsersState = {
    /** Store a user to ask for confirmation before deleting him */
    confirmDelete: GetUser | null;

    /** Toggle the modal form for creating new users*/
    createNew: boolean;

    // queried data
    users?: Array<GetUser>;

    // controlled state
    /** New user's display name */
    newDisplay: string;
    /** New user's username */
    newName: string;
    /** New user's password */
    newPwd: string;
    /** New user's admin flag */
    newAdmin: boolean;
};

/** View to expose the `/api/v1/admin/users` endpoints */
export default class AdminUsers extends React.Component<AdminUsersProps, AdminUsersState> {
    state: AdminUsersState = {
        confirmDelete: null,
        createNew: false,
        newDisplay: "",
        newName: "",
        newPwd: "",
        newAdmin: false,
    };

    componentDidMount() {
        this.fetchState();
    }

    fetchState() {
        Api.admin.users.all().then(
            handleApiError(({ users }) =>
                this.setState({
                    users,
                }),
            ),
        );
    }

    render() {
        const { users } = this.state;
        if (users === undefined) return <Loading />;

        return (
            <div className="pane">
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Username</th>
                            <th>Display name</th>
                            <th>Created At</th>
                            <th>Last Login</th>
                            <th>Admin</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {users.map((user) => (
                            <tr key={user.uuid}>
                                <td>{user.uuid}</td>
                                <td>{user.username}</td>
                                <td>{user.displayName}</td>
                                <td>{user.createdAt.toLocaleString()}</td>
                                <td>{(user.lastLogin && user.lastLogin.toLocaleString()) || "never"}</td>
                                <td>
                                    <input
                                        type="checkbox"
                                        checked={user.permission === UserPermission.Admin}
                                        disabled
                                    />
                                </td>
                                <td>
                                    <button
                                        className="button"
                                        type="button"
                                        onClick={() => this.setState({ confirmDelete: user })}
                                    >
                                        Delete
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
                <button type="button" className="button" onClick={() => this.setState({ createNew: true })}>
                    Create new user
                </button>
                <Popup
                    modal
                    nested
                    open={this.state.confirmDelete != null}
                    onClose={() => this.setState({ confirmDelete: null })}
                >
                    <div className="popup-content pane">
                        <h1 className="heading neon">
                            Are you sure you want to delete {this.state.confirmDelete?.displayName || ""}?
                        </h1>
                        <button className="button" type="button" onClick={() => this.deleteUser()}>
                            Delete
                        </button>
                        <button className="button" type="button" onClick={() => this.setState({ confirmDelete: null })}>
                            Chancel
                        </button>
                    </div>
                </Popup>
                <Popup modal nested open={this.state.createNew} onClose={() => this.setState({ createNew: false })}>
                    <form
                        className="popup-content pane"
                        onSubmit={(e) => {
                            e.preventDefault();
                            this.createUser();
                        }}
                    >
                        <label>Username:</label>
                        <Input value={this.state.newName} onChange={(newName) => this.setState({ newName })} />
                        <label>Display Name:</label>
                        <Input value={this.state.newDisplay} onChange={(newDisplay) => this.setState({ newDisplay })} />
                        <label>Password:</label>
                        <Input
                            type="password"
                            value={this.state.newPwd}
                            onChange={(newPwd) => this.setState({ newPwd })}
                        />
                        <label>Admin:</label>
                        <input
                            type="checkbox"
                            checked={this.state.newAdmin}
                            onChange={(e) => this.setState({ newAdmin: e.target.checked })}
                        />
                        <button type="submit" className="button">
                            Create
                        </button>
                    </form>
                </Popup>
            </div>
        );
    }

    deleteUser() {
        const { confirmDelete } = this.state;
        if (confirmDelete === null) return;

        Api.admin.users.delete(confirmDelete.uuid).then(
            handleApiError(() => {
                toast.success("Deleted user");
                this.setState({ confirmDelete: null });
                this.fetchState();
            }),
        );
    }

    createUser() {
        const { newName, newDisplay, newPwd, newAdmin } = this.state;

        if (
            !check([
                [newName.length > 0, "Empty username"],
                [newDisplay.length > 0, "Empty display name"],
                [newPwd.length > 0, "Empty password"],
            ])
        )
            return;

        Api.admin.users
            .create({
                username: newName,
                displayName: newDisplay,
                password: newPwd,
                permission: newAdmin ? UserPermission.Admin : UserPermission.Default,
            })
            .then(
                handleApiError(() => {
                    toast.success("Created user");
                    this.setState({ createNew: false, newName: "", newDisplay: "", newPwd: "" });
                    this.fetchState();
                }),
            );
    }
}
