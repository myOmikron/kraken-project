import React from "react";

type LoadingProps = {};

/**
 * Simple component indicating that a resource is still loading
 *
 * TODO: make it pretty
 */
export default function Loading(props: LoadingProps) {
    return <div className="loading">"Loading..."</div>;
}
