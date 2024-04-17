import React, { forwardRef } from "react";

/** React props for [`<Checkbox />`]{@link Checkbox} */
export type CheckboxProps = {
    /** Is the checkbox selected? */
    value: boolean;
    /** Callback invoked when the user changed the checkbox */
    onChange: (newValue: boolean) => void;
    /** Should this component capture the focus after mounting? */
    autoFocus?: boolean;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "autoFocus" | "type">;

/**
 * A basic `<input type="checkbox" />`
 *
 * This wrapper implements a [controlled component]{@link https://react.dev/learn/sharing-state-between-components#controlled-and-uncontrolled-components}.
 * It also applies the correct css class.
 */
export const Checkbox = forwardRef((props: CheckboxProps, ref) => {
    const { value, onChange, autoFocus, ...passThrough } = props;

    const callback = React.useCallback((element: HTMLInputElement) => {
        if (element && autoFocus) {
            setTimeout(function () {
                element.focus();
            }, 10);
        }

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
