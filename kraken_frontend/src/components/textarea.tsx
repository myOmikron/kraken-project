import React from "react";

/** React props for [`<Textarea />`]{@link Textarea} */
export type TextareaProps = {
    /** The text area's current value */
    value: string;
    /** Callback invoked when the user changed the value */
    onChange: (newValue: string) => void;
    /** Should this component capture the focus after mounting? */
    autoFocus?: boolean;
} & Omit<React.TextareaHTMLAttributes<HTMLTextAreaElement>, "value" | "onChange" | "autoFocus">;

/**
 * A basic `<textarea />`
 *
 * This wrapper implements a [controlled component]{@link https://react.dev/learn/sharing-state-between-components#controlled-and-uncontrolled-components}.
 * It also applies the correct css class.
 */
export default function Textarea(props: TextareaProps) {
    const { value, onChange, autoFocus, ...passThrough } = props;

    const callback = React.useCallback((element: HTMLTextAreaElement) => {
        if (element && autoFocus) {
            setTimeout(function () {
                element.focus();
            }, 10);
        } // eslint-disable-next-line
    }, []);

    return (
        <textarea
            className="textarea"
            value={value}
            onChange={(e) => {
                onChange(e.target.value);
            }}
            ref={callback}
            {...passThrough}
        />
    );
}
