import React from "react";

/** React props for [`<Input />`]{@link Input} */
export type InputProps = {
    /** The input's current value */
    value: string;
    /** Callback invoked when the user changed the value */
    onChange: (newValue: string) => void;
    /** Should this component capture the focus after mounting? */
    autoFocus?: boolean;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "autoFocus">;

// TODO: lint should catch this
const Input = React.forwardRef<HTMLInputElement, InputProps>(function (props, ref) {
    const { value, onChange, autoFocus, ...passThrough } = props;

    const internalRef = React.createRef<HTMLInputElement>();
    React.useImperativeHandle<HTMLInputElement | null, HTMLInputElement | null>(ref, () => internalRef.current, [
        internalRef.current,
    ]);

    React.useEffect(() => {
        if (autoFocus) {
            setTimeout(function () {
                if (internalRef.current !== null) internalRef.current.focus();
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
            ref={internalRef}
            {...passThrough}
        />
    );
});
export default Input;
