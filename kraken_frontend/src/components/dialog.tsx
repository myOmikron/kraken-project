// TODO: CSS

import React from "react";
import { createRoot } from "react-dom/client";
import Input from "./input";

export type DialogProps = {
    /**
     * Title shown as header of dialog.
     */
    title: string;
    /**
     * Mapping submit|cancel|reset -> UI human readable string.
     *
     * Buttons are only shown if they are non-undefined and may show default
     * strings when empty.
     */
    buttons: {
        submit?: string,
        cancel?: string,
        reset?: string,
    };
    /**
     * Modal dialogs are rendered on a separate layer and prevent user input,
     * regular dialogs allow clicking elsewhere still.
     */
    modal?: boolean;
    /**
     * Set to true to show, set to false to hide - may be changed at any point.
     * Setting to false cancels the dialog and is the same as calling `cancel`.
     * Setting to true is the same as calling `show`.
     */
    visible?: boolean;
    onClose?: (reason: DialogCloseReason) => any;
    children?: React.ReactNode;
};

export type DialogCloseReason = "other" | "submit" | "cancel";

export default class Dialog extends React.Component<DialogProps> {
    dialogRef: React.RefObject<HTMLDialogElement>;
    formRef: React.RefObject<HTMLFormElement>;

    constructor(props: DialogProps) {
        super(props);
        this.dialogRef = React.createRef();
        this.formRef = React.createRef();
        this.state = {};
    }

    componentDidMount(): void {
        if (this.props.visible)
            this.show();
    }

    componentDidUpdate(pProps: DialogProps) {
        if (this.props.visible != pProps.visible) {
            if (this.props.visible)
                this.show();
            else
                this.cancel();
        }
    }

    show() {
        if (this.dialogRef.current) {
            this.dialogRef.current.returnValue = "";
            if (this.props.modal)
                this.dialogRef.current.showModal();
            else
                this.dialogRef.current.show();
        }
    }

    cancel() {
        if (this.dialogRef.current && this.formRef.current) {
            this.formRef.current.reset();
            this.dialogRef.current.close("cancel");
        }
    }

    reset() {
        if (this.formRef.current) {
            this.formRef.current.reset();
        }
    }

    handleClose() {
        if (this.props.onClose) {
            let dialog = this.dialogRef.current;
            if (!dialog)
                throw new Error("dialogRef is unset!");
            let reason: DialogCloseReason = "other";
            if (dialog.returnValue === "cancel"
                || dialog.returnValue === "submit")
                reason = dialog.returnValue;
            this.props.onClose(reason);
        }
    }

    render() {
        return (
            <dialog ref={this.dialogRef} onClose={this.handleClose.bind(this)}>
                <form method="dialog" ref={this.formRef}>
                    <header>
                        <h2>{this.props.title}</h2>
                    </header>
                    <main>
                        {this.props.children}
                    </main>
                    <footer>
                        {this.props.buttons.reset !== undefined && <button type="reset">{this.props.buttons.reset || "Reset"}</button>}
                        {this.props.buttons.cancel !== undefined && <button type="reset" onClick={this.cancel}>{this.props.buttons.cancel || "Cancel"}</button>}
                        {this.props.buttons.submit !== undefined && <button type="submit" value="submit">{this.props.buttons.submit || "Submit"}</button>}
                    </footer>
                </form>
            </dialog>
        );
    }
}

export async function promptInput(title: string, defaultInput?: string): Promise<string | undefined> {
    let holder = document.body.appendChild(document.createElement("div"));
    let root = createRoot(holder);
    return new Promise<string | undefined>((resolve) => {
        class InputDialog extends React.Component<{
            defaultValue: string | undefined
        }, {
            value: string
        }> {
            constructor(props: any) {
                super(props);
                this.state = {
                    value: props.defaultValue || ""
                };
            }

            render() {
                return (<Dialog
                    modal
                    visible
                    title={title}
                    buttons={{ submit: "Ok" }}
                    onClose={(reason) => {
                        if (reason == "submit")
                            resolve(this.state.value);
                        else
                            resolve(undefined);
                    }}
                >
                    <Input autoFocus value={this.state.value} onChange={(newV) => {
                        this.setState({
                            value: newV
                        });
                    }} />
                </Dialog>);
            }
        }

        root.render(
            <InputDialog defaultValue={defaultInput} />
        );
    }).finally(() => {
        root.unmount();
        document.body.removeChild(holder);
    });
}

export async function alertReact(title: string, content?: React.ReactNode): Promise<undefined> {
    let holder = document.body.appendChild(document.createElement("div"));
    let root = createRoot(holder);
    return new Promise<undefined>((resolve) => {
        class AlertDialog extends React.Component {
            constructor(props: any) {
                super(props);
            }

            render() {
                return (<Dialog
                    modal
                    visible
                    title={title}
                    buttons={{ submit: "Ok" }}
                    onClose={(reason) => {
                        resolve(undefined);
                    }}
                >
                    {content}
                </Dialog>);
            }
        }

        root.render(
            <AlertDialog />
        );
    }).finally(() => {
        root.unmount();
        document.body.removeChild(holder);
    });
}

export async function alertText(title: string, content?: string): Promise<undefined> {
    return alertReact(title, content && (<p>{content}</p>));
}
