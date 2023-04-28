import React from "react";
import { USER_CONTEXT } from "../context/user";
import { ROUTES } from "../routes";
import { toast } from "react-toastify";

export type AdminGuardProps = {
    children: React.ReactNode;
};

/** Wrapper for views to make them only visible for admins */
export default function AdminGuard(props: AdminGuardProps) {
    const { user } = React.useContext(USER_CONTEXT);
    if (user.admin) return <>{props.children}</>;
    else {
        toast.warning("You don't have the required permissions");
        ROUTES.HOME.visit({});
        return null;
    }
}
