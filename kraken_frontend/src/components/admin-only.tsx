import React from "react";
import { UserPermission } from "../api/generated";
import USER_CONTEXT from "../context/user";

/** React props for [`<AdminOnly />`]{@link AdminOnly} */
export type AdminOnlyProps = {
    /** Content to hide for non-admin users */
    children: React.ReactNode;
};

/** Hide's its content from non-admin users */
export function AdminOnly(props: AdminOnlyProps) {
    const { user } = React.useContext(USER_CONTEXT);
    if (user.permission === UserPermission.Admin) return <>{props.children}</>;
    else return undefined;
}
