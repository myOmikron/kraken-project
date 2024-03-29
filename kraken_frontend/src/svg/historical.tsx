import React from "react";

export default function HistoricalIcon(props: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div className={"icon"} {...props}>
            <svg fill="#000000" className="neon" viewBox="0 0 256 256" id="Flat" xmlns="http://www.w3.org/2000/svg">
                <path d="M204,75.64111V40a20.02229,20.02229,0,0,0-20-20H72A20.02229,20.02229,0,0,0,52,40V76a20.09552,20.09552,0,0,0,8,16l48,36L60,164a20.09552,20.09552,0,0,0-8,16v36a20.02229,20.02229,0,0,0,20,20H184a20.02229,20.02229,0,0,0,20-20V180.35889a20.10312,20.10312,0,0,0-7.93994-15.95459L147.90039,128l48.16016-36.4043A20.10408,20.10408,0,0,0,204,75.64111ZM180,212H76V182l51.97168-38.97852L180,182.34961Zm0-138.34961-52.02832,39.32813L76,74V44H180Z" />
            </svg>
        </div>
    );
}
