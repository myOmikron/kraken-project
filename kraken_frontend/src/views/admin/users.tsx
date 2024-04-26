import React from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api } from "../../api/api";
import { FullUser, UserPermission } from "../../api/generated";
import Input from "../../components/input";
import Loading from "../../components/loading";
import { check, handleApiError } from "../../utils/helper";

/**
 * View to expose the `/api/v1/admin/users` endpoints
 *
 * @returns JSX.Element
 */
export default function AdminUsers() {
    /** Store a user to ask for confirmation before deleting him */
    const [confirmDelete, setConfirmDelete] = React.useState<FullUser | null>(null);

    /** Toggle the modal form for creating new users*/
    const [createNew, setCreateNew] = React.useState<boolean>(false);

    // queried data
    const [users, setUsers] = React.useState<Array<FullUser> | undefined>(undefined);

    // controlled state
    /** New user's display name */
    const [newDisplay, setNewDisplay] = React.useState<string>("");
    /** New user's username */
    const [newName, setNewName] = React.useState<string>("");
    /** New user's password */
    const [newPwd, setNewPwd] = React.useState<string>("");
    /** New user's admin flag */
    const [newAdmin, setNewAdmin] = React.useState<boolean>(false);

    /** sends api request to fetch all users and sets the state*/
    function fetchUser() {
        Api.admin.users.all().then(
            handleApiError(({ users }) => {
                setUsers(users);
            }),
        );
    }

    React.useEffect(() => {
        fetchUser();
    }, []);

    /** sends api request to delete one confirmed user */
    function deleteUser() {
        if (confirmDelete === null) return;

        Api.admin.users.delete(confirmDelete.uuid).then(
            handleApiError(() => {
                toast.success("Deleted user");
                setConfirmDelete(null);
                fetchUser();
            }),
        );
    }

    /** send api request to create a new User */
    function createUser() {
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
                    setCreateNew(false);
                    setNewName("");
                    setNewDisplay("");
                    setNewPwd("");
                    fetchUser();
                }),
            );
    }

    return (
        <>
            {users === undefined ? (
                <Loading />
            ) : (
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
                                        <button className="button" type="button" onClick={() => setConfirmDelete(user)}>
                                            Delete
                                        </button>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    <button type="button" className="button" onClick={() => setCreateNew(true)}>
                        Create new user
                    </button>
                    <Popup modal nested open={confirmDelete !== null} onClose={() => setConfirmDelete(null)}>
                        <div className="popup-content pane">
                            <h1 className="heading neon">
                                Are you sure you want to delete {confirmDelete?.displayName || ""}?
                            </h1>
                            <button className="button" type="button" onClick={() => deleteUser()}>
                                Delete
                            </button>
                            <button className="button" type="button" onClick={() => setConfirmDelete(null)}>
                                Cancel
                            </button>
                        </div>
                    </Popup>
                    <Popup modal nested open={createNew} onClose={() => setCreateNew(false)}>
                        <form
                            className="popup-content pane"
                            onSubmit={(e) => {
                                e.preventDefault();
                                createUser();
                            }}
                        >
                            <label>Username:</label>
                            <Input value={newName} onChange={(newName) => setNewName(newName)} />
                            <label>Display Name:</label>
                            <Input value={newDisplay} onChange={(newDisplay) => setNewDisplay(newDisplay)} />
                            <label>Password:</label>
                            <Input type="password" value={newPwd} onChange={(newPwd) => setNewPwd(newPwd)} />
                            <label>Admin:</label>
                            <input type="checkbox" checked={newAdmin} onChange={(e) => setNewAdmin(e.target.checked)} />
                            <button type="submit" className="button">
                                Create
                            </button>
                        </form>
                    </Popup>
                </div>
            )}
        </>
    );
}
