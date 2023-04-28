import React from "react";

export type TextareaProps = {
    value: string;
    onChange: (newValue: string) => any;
    autoFocus?: boolean;
} & Omit<React.TextareaHTMLAttributes<HTMLTextAreaElement>, "value" | "onChange" | "autoFocus">;

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
