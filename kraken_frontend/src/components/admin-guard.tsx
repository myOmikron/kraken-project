import React from "react";
import { toast } from "react-toastify";
import { UserPermission } from "../api/generated";
import USER_CONTEXT from "../context/user";
import { ROUTES } from "../routes";

/** React props for [`<AdminGuard />`]{@link AdminGuard} */
export type AdminGuardProps = {
    /** View only accessible by admins */
    children: React.ReactNode;
};

/** Guard "protecting" views by redirecting non admins to "/" */
export default function AdminGuard(props: AdminGuardProps) {
    const { user } = React.useContext(USER_CONTEXT);
    if (user.permission === UserPermission.Admin) return <>{props.children}</>;
    else {
        toast.warning("You don't have the required permissions");
        ROUTES.HOME.visit({});
        return null;
    }
}
