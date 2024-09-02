import React, { HTMLProps, ReactNode, useEffect } from "react";
import "../../../styling/collapsible-section.css";
import ArrowDownIcon from "../../../svg/arrow-down";

/** React props for [`<CollapsibleSection />`]{@link CollapsibleSection} */
export type CollapsibleSectionProps = {
    /**
     * The content to render inside the heading before the arrow shaped button
     */
    summary: ReactNode;

    /**
     * Should the section start a visible or collapsed?
     *
     * This property is only read once.
     */
    defaultVisible?: boolean;

    /**
     * Is the section currently visible or collapsed?
     *
     * Leave this prop empty, to let the component manage its own state.
     */
    visible?: boolean;

    /**
     * Callback to be notified, when the user wants to change the sections visibility
     */
    onChangeVisible?: (visible: boolean) => void;
} & Omit<HTMLProps<HTMLDivElement>, "summary">;

/**
 * A div with a heading and an arrow shaped button to toggle its children's visibility
 *
 * This component can be used as controlled and uncontrolled component.
 */
export default function CollapsibleSection(props: CollapsibleSectionProps) {
    const { summary, children, defaultVisible, visible: wantVisible, onChangeVisible, ...passThrough } = props;
    const [visible, setVisible] = React.useState(defaultVisible ?? true);

    useEffect(() => {
        if (wantVisible !== undefined) setVisible(wantVisible);
    }, [wantVisible]);

    return (
        <div className="collapsible-section" {...passThrough}>
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
