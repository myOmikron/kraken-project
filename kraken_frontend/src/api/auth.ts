import { toast } from "react-toastify";

import { Err, Ok, Result } from "../utils/result";
import { ApiError, StatusCode, parseError } from "./error";
import { headers } from "./helper";

export async function test(): Promise<"logged out" | "logged in" | "verified"> {
    const res = await fetch("/api/v1/auth/test");
    if (res.status === 200) {
        return "verified";
    } else {
        const error = await parseError(res);
        switch (error.status_code) {
            case StatusCode.Unauthenticated:
                return "logged out";
            case StatusCode.Missing2fa:
                return "logged in";
            default:
                toast.error(error.message);
                return "logged out";
        }
    }
}

export async function login(username: string, password: string): Promise<Result<null, ApiError>> {
    const res = await fetch("/api/v1/auth/login", {
        method: "post",
        body: JSON.stringify({ username: username, password: password }),
        headers,
    });
    if (res.status === 200) {
        return Ok(null);
    } else {
        return Err(await parseError(res));
    }
}

export async function logout(): Promise<Result<null, ApiError>> {
    const res = await fetch("/api/v1/auth/logout", {
        method: "get",
        headers,
    });
    if (res.status === 200) {
        return Ok(null);
    } else {
        return Err(await parseError(res));
    }
}
