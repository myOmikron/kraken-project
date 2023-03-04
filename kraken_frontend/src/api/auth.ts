// @ts-nocheck
import { Base64 } from "js-base64";
import { toast } from "react-toastify";

import { Err, Ok, Result } from "../utils/result";
import { headers } from "./helper";
import { ApiError, StatusCode, parseError } from "./error";

type BrowserError = {
    success: false;
    message: string;
};

export async function test(): Promise<"logged out" | "logged in" | "verified"> {
    const res = await fetch("/api/v1/auth/test");
    if (res.status === 200) {
        return "verified";
    } else {
        const error = await parseError(res);
        switch (error.status_code) {
            case StatusCode.Unauthenticated:
                return "logged out";
            case StatusCode.Missing2FA:
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

export async function authenticate(): Promise<Result<null, ApiError | BrowserError>> {
    let res;
    res = await fetch("/api/v1/auth/start_auth", {
        method: "post",
        headers,
    });
    if (res.status !== 200) {
        return Err(await parseError(res));
    }

    const challenge = await res.json();
    const { publicKey } = challenge;
    console.debug("Received challenge from server", challenge);

    const credential = await navigator.credentials.get({
        publicKey: {
            rpId: publicKey.rpId,
            timeout: publicKey.timeout,
            userVerification: publicKey.userVerification,
            challenge: Base64.toUint8Array(publicKey.challenge),
            allowCredentials: publicKey.allowCredentials.map((c) => {
                return {
                    id: Base64.toUint8Array(c.id),
                    type: c.type,
                };
            }),
        },
    });
    console.debug("Received credential from browser", credential);
    if (credential === null) {
        return Err({ success: false, message: "Getting credentials from browser failed" });
    }

    res = await fetch("/api/v1/auth/finish_auth", {
        method: "post",
        body: JSON.stringify({
            id: Base64.fromUint8Array(new Uint8Array(credential.id), true),
            rawId: Base64.fromUint8Array(new Uint8Array(credential.rawId), true),
            response: {
                authenticatorData: Base64.fromUint8Array(new Uint8Array(credential.response.authenticatorData), true),
                clientDataJSON: Base64.fromUint8Array(new Uint8Array(credential.response.clientDataJSON), true),
                signature: Base64.fromUint8Array(new Uint8Array(credential.response.signature), true),
                userHandle: credential.response.userHandle,
            },
            type: credential.type,
        }),
        headers,
    });
    if (res.status === 200) {
        return Ok(null);
    } else {
        return Err(await parseError(res));
    }
}

export async function registerKey(): Promise<Result<null, ApiError | BrowserError>> {
    let res;
    res = await fetch("/api/v1/auth/start_register", {
        method: "post",
        headers,
    });
    if (res.status !== 200) {
        return Err(await parseError(res));
    }

    const challenge = await res.json();
    const { publicKey } = challenge;
    console.debug("Received challenge from server", challenge);

    const credential = await navigator.credentials.create({
        publicKey: {
            user: {
                id: Base64.toUint8Array(publicKey.user.id),
                name: publicKey.user.name,
                displayName: publicKey.user.displayName,
            },
            attestation: publicKey.attestation,
            rp: publicKey.rp,
            challenge: Base64.toUint8Array(publicKey.challenge),
            extensions: publicKey.extensions,
            timeout: publicKey.timeout,
            excludeCredentials: publicKey.excludeCredentials.map((c) => {
                return {
                    id: Base64.toUint8Array(c.id),
                    type: c.type,
                };
            }),
            pubKeyCredParams: publicKey.pubKeyCredParams,
            authenticatorSelection: publicKey.authenticatorSelection,
        },
    });
    console.debug("Created credential from browser", credential);
    if (credential === null) {
        return Err({ success: false, message: "Requesting new credentials from browser failed" });
    }

    res = await fetch("/api/v1/auth/finish_register", {
        method: "post",
        body: JSON.stringify({
            id: credential.id,
            rawId: Base64.fromUint8Array(new Uint8Array(credential.rawId), true),
            response: {
                attestationObject: Base64.fromUint8Array(new Uint8Array(credential.response.attestationObject), true),
                clientDataJSON: Base64.fromUint8Array(new Uint8Array(credential.response.clientDataJSON), true),
            },
            type: credential.type,
        }),
        headers,
    });
    if (res.status !== 200) {
        return Err(await parseError(res));
    } else {
        return Ok(null);
    }
}
