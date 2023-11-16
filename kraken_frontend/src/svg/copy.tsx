import React from "react";

export default function CopyIcon(props: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div className={"icon"} {...props}>
            <svg
                className={"neon"}
                width="512px"
                height="512px"
                viewBox="0 0 512 512"
                xmlns="http://www.w3.org/2000/svg"
            >
                <title>ionicons-v5-e</title>
                <path d="M408,480H184a72,72,0,0,1-72-72V184a72,72,0,0,1,72-72H408a72,72,0,0,1,72,72V408A72,72,0,0,1,408,480Z" />
                <path d="M160,80H395.88A72.12,72.12,0,0,0,328,32H104a72,72,0,0,0-72,72V328a72.12,72.12,0,0,0,48,67.88V160A80,80,0,0,1,160,80Z" />
            </svg>
        </div>
    );
}
