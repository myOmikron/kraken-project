import React, { forwardRef } from "react";

export type CheckboxProps = {
    value: boolean;
    onChange: (newValue: boolean) => any;
    autoFocus?: boolean;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "autoFocus" | "type">;

export const Checkbox = forwardRef((props: CheckboxProps, ref) => {
    const { value, onChange, autoFocus, ...passThrough } = props;

    const callback = React.useCallback((element: HTMLInputElement) => {
        if (element && autoFocus) {
            setTimeout(function () {
                element.focus();
            }, 10);
        } // eslint-disable-next-line

        if (typeof ref == "function") ref(element);
        else if (ref) ref.current = element;
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
});
export default Checkbox;
