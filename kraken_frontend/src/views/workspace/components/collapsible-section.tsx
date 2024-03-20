import React, { HTMLProps, ReactNode, useEffect } from "react";
import "../../../styling/collapsible-section.css";
import ArrowDownIcon from "../../../svg/arrow-down";

export type CollapsibleSectionProps = {
    summary: ReactNode;
    defaultVisible?: boolean;
    visible?: boolean;
    onChangeVisible?: (visible: boolean) => void;
} & Omit<HTMLProps<HTMLDivElement>, "summary">;

export default function CollapsibleSection({
    summary,
    children,
    defaultVisible,
    visible: wantVisible,
    onChangeVisible,
    ...props
}: CollapsibleSectionProps) {
    const [visible, setVisible] = React.useState(defaultVisible ?? true);

    useEffect(() => {
        if (wantVisible !== undefined) setVisible(wantVisible);
    }, [wantVisible]);

    return (
        <div className="collapsible-section" {...props}>
            <h2 className={"sub-heading"}>
                {summary}
                <div
                    className="toggle"
                    onClick={() => {
                        onChangeVisible?.(!visible);
                        if (wantVisible === undefined) setVisible((v) => !v);
                    }}
                >
                    <ArrowDownIcon inverted={visible} />
                </div>
            </h2>
            {visible && children}
        </div>
    );
}
