import React from "react";

export type CheckboxProps = {
    value: boolean;
    onChange: (newValue: boolean) => any;
    autoFocus?: boolean;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "autoFocus" | "type">;

export default function Checkbox(props: CheckboxProps) {
    const { value, onChange, autoFocus, ...passThrough } = props;

    const callback = React.useCallback((element: HTMLInputElement) => {
        if (element && autoFocus) {
            setTimeout(function () {
                element.focus();
            }, 10);
        } // eslint-disable-next-line
    }, []);

    return (
        <input
            className="checkbox"
            type="checkbox"
            checked={value}
            onChange={(e) => {
                onChange(e.target.checked);
            }}
            ref={callback}
            {...passThrough}
        />
    );
}
