import React from "react";

export type InputProps = {
    value: string;
    onChange: (newValue: string) => void;
    autoFocus?: boolean;
} & Omit<React.InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "autoFocus">;

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
