import React from "react";

export type InputProps = {
    value: string;
    onChange: (newValue: string) => any;
    autoFocus?: boolean;
} & any;

export default function Input(props: InputProps) {
    const { value, onChange, autoFocus, onEnter, ...otherProps } = props;

    const callback = React.useCallback((element: HTMLInputElement) => {
        if (element && autoFocus) {
            setTimeout(function () {
                element.focus();
            }, 10);
        } // eslint-disable-next-line
    }, []);

    return (
        <input
            className="input"
            value={value}
            onChange={(e) => {
                onChange(e.target.value);
            }}
            ref={callback}
            {...otherProps}
        />
    );
}
