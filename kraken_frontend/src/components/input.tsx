import React from "react";

export type InputProps = {
    as?: "input" | "textarea";
    value: string;
    onChange: (newValue: string) => any;
    autoFocus?: boolean;
    /** for `as == textarea`, number of rows before resizing */
    rows?: number;
} & Omit<React.InputHTMLAttributes<HTMLInputElement | HTMLTextAreaElement>, "value" | "onChange" | "autoFocus">;

const Input = React.forwardRef<HTMLInputElement, InputProps>(function (props, ref) {
    const { as, value, onChange, autoFocus, rows, ...passThrough } = props;

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

    return as == "textarea" ? (
        <textarea
            className="input textarea"
            value={value}
            onChange={(e) => {
                onChange(e.target.value);
            }}
            rows={rows}
            ref={internalRef as any}
            {...passThrough}
        />
    ) : (
        <input
            className="input"
            value={value}
            onChange={(e) => {
                onChange(e.target.value);
            }}
            ref={internalRef as any}
            {...passThrough}
        />
    );
});
export default Input;
