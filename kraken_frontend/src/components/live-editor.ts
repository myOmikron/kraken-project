import { editor } from "monaco-editor";
import { EditorTarget, SimpleUser } from "../api/generated";
import React from "react";
import Cursor from "../utils/monaco-cursor";
import USER_CONTEXT from "../context/user";
import { useStableObj } from "../utils/helper";
import WS from "../api/websocket";
import { Monaco, useMonaco } from "@monaco-editor/react";
import { toast } from "react-toastify";

/**
 * Arguments to the {@link useLiveEditor} hook
 */
export type UseLiveEditorArgs<CT extends {} = true> = {
    /**
     * The editor to sync (might be `null` on initial render)
     */
    editorInstance: editor.IStandaloneCodeEditor | null;

    /**
     * The {@link EditorTarget} to use when sending updates to the server
     */
    target: EditorTarget;

    /**
     * Filter an incoming cursor update for the right target and extract it
     *
     * Return `undefined` if the update should be ignored (i.e. you may omit the return statement)
     */
    receiveCursor: (target: EditorTarget) => CT | undefined;

    /**
     * An optional list which triggers a deletion of all cursors when any member changes
     *
     * I.e. it is used as argument to a {@link React.useEffect `React.useEffect`} call which deletes the cursors
     */
    deleteCursors?: React.DependencyList;

    /**
     * An optional list which triggers a re-evaluation of `isCursorHidden` for all cursors when any member changes
     *
     * I.e. it is used as argument to a {@link React.useEffect `React.useEffect`} call which re-evaluates the cursors
     *
     * _Should be set in combination with `isCursorHidden`_
     */
    hideCursors?: React.DependencyList;

    /**
     * Optional filter to hide certain cursors
     *
     * If it is not specified, all cursors will be visible.
     *
     * _Should be set in combination with `hideCursors`_
     */
    isCursorHidden?: (cursor: SimpleUser & CT) => boolean;

    /**
     * Optional flag to include the own cursor
     *
     * Normally the user's own cursor won't be included.
     */
    includeOwnCursor?: boolean;

    /**
     * Filter an incoming edit update for the right target and return the model it applies to
     *
     * Return `undefined` if the update should be ignored (i.e. you may omit the return statement)
     */
    receiveEdit: (
        target: EditorTarget,
        editor: editor.IStandaloneCodeEditor | null,
        monaco: Monaco,
    ) => { model: editor.ITextModel; setValue?: (newValue: string) => void } | undefined;

    /**
     * The React setter for the value currently displayed in the editor
     */
    setValue: (newValue: string) => void;

    /**
     * Optional callback invoked with the entire text when the user performs an edit
     *
     * This differs from the `<Editor />`'s `onChange` which is also invoked for other users' edits.
     */
    onUserEdit?: (newValue: string) => void;
};

export default function useLiveEditor<CT extends {} = true>(args: UseLiveEditorArgs<CT>) {
    const {
        editorInstance,
        target,
        receiveCursor,
        deleteCursors = [],
        hideCursors = [],
        isCursorHidden,
        includeOwnCursor = false,
        receiveEdit,
        setValue,
        onUserEdit,
    } = args;
    const monaco = useMonaco();
    const { user } = React.useContext(USER_CONTEXT);
    const stableArgs = useStableObj({
        target,
        receiveCursor,
        isCursorHidden,
        includeOwnCursor,
        receiveEdit,
        setValue,
        user: user.uuid,
        onUserEdit,
    });

    /*
     * CURSORS
     */

    type Cursors = Record<string, { data: SimpleUser & CT; cursor: Cursor }>;

    const [cursors, setCursors] = React.useState<Cursors>({});

    // Delete cursors
    React.useEffect(
        () => () =>
            setCursors((cursors) => {
                for (const { cursor } of Object.values(cursors)) {
                    cursor.delete();
                }
                return {};
            }),
        deleteCursors,
    );

    // Update which cursors to show based on `isHidden`
    React.useEffect(
        () =>
            setCursors((cursors) => {
                for (const { cursor, data } of Object.values(cursors)) {
                    if (isCursorHidden === undefined) {
                        cursor.updateActive(true);
                    } else {
                        cursor.updateActive(!isCursorHidden(data));
                    }
                }
                return { ...cursors };
            }),
        hideCursors,
    );

    React.useEffect(() => {
        // Pass the editor to cursors which have been created before the editor was loaded
        setCursors((cursors) => {
            for (const { cursor } of Object.values(cursors)) {
                cursor.updateEditor(editorInstance);
            }
            return { ...cursors };
        });

        // Send outgoing cursor messages
        let disposable = {
            dispose() {},
        };
        if (editorInstance !== null) {
            editorInstance.onDidChangeCursorSelection;
            disposable = editorInstance.onDidChangeCursorPosition((event) => {
                WS.send({
                    type: "EditorChangedCursor",
                    target: stableArgs.target,
                    cursor: {
                        line: event.position.lineNumber,
                        column: event.position.column,
                    },
                });
            });
        }

        // Save incoming cursors messages
        const handle = WS.addEventListener("message.EditorChangedCursor", (event) => {
            if (!stableArgs.includeOwnCursor && stableArgs.user === event.user.uuid) return;

            const cursorTarget = stableArgs.receiveCursor(event.target);
            if (cursorTarget === undefined) return;

            const id = event.user.uuid;
            const { line, column } = event.cursor;
            const data = { ...event.user, ...cursorTarget };

            setCursors((cursors) => {
                let cursor;
                if (id in cursors) {
                    cursor = cursors[id].cursor;
                    cursor.updatePosition(line, column);
                    if (stableArgs.isCursorHidden !== undefined) cursor.updateActive(!stableArgs.isCursorHidden(data));
                } else {
                    cursor = new Cursor(
                        editorInstance,
                        line,
                        column,
                        stableArgs.isCursorHidden && !stableArgs.isCursorHidden(data),
                    );
                }
                return {
                    ...cursors,
                    [id]: { cursor, data },
                };
            });
        });

        return () => {
            WS.removeEventListener(handle);
            disposable.dispose();
        };
    }, [editorInstance]);

    /*
     * EDITS
     */

    // Boolean flag which indicates whether changes to the editor should be sent over the websocket
    //
    // When we apply changed we received from the websocket, they will trigger the `onChange` handler.
    // But those events shouldn't be sent as the user's edits. So this flag can disable the "send changes to ws"-part.
    // This is stored a ref instead of state because it needs to be changed in between renders.
    const sendChanges = React.useRef(true);

    // Save incoming edit messages
    React.useEffect(() => {
        const handle = WS.addEventListener("message.EditorChangedContent", (event) => {
            if (event.user.uuid === user.uuid) return; // TODO: might need more consideration

            if (monaco === null) {
                toast.error("Monaco is not loaded yet. Please report this error to the devs"); // TODO
                return;
            }

            const tmp = stableArgs.receiveEdit(event.target, editorInstance, monaco);
            if (tmp === undefined) return;
            const { model, setValue } = tmp;

            // Disable sending edit events
            sendChanges.current = false;

            const { text, startColumn, startLine, endColumn, endLine } = event.change;
            model.applyEdits([
                { range: { startColumn, endColumn, startLineNumber: startLine, endLineNumber: endLine }, text },
            ]);

            // Re-enable sending edit events
            sendChanges.current = true;

            if (setValue !== undefined) setValue(model.getValue());
        });
        return () => WS.removeEventListener(handle);
    }, [editorInstance, user.uuid]);

    // Send outgoing edit messages
    //
    // This function is passed to `<Editor onChange={...} />`.
    // We use `useCallback` because `<Editor />` uses it internally in a dependency list.
    const onChange = React.useCallback((value: string | undefined, event: editor.IModelContentChangedEvent) => {
        // Update the React state
        if (value !== undefined) stableArgs.setValue(value);

        // Send the changes to the websocket
        if (sendChanges.current) {
            if (value !== undefined && stableArgs.onUserEdit) {
                stableArgs.onUserEdit(value);
            }
            for (const change of event.changes) {
                const {
                    text,
                    range: { startColumn, startLineNumber, endLineNumber, endColumn },
                } = change;
                WS.send({
                    type: "EditorChangedContent",
                    target: stableArgs.target,
                    change: { text, startColumn, endColumn, startLine: startLineNumber, endLine: endLineNumber },
                });
            }
        }
    }, []);

    /*
     * RETURN
     */

    return { cursors: Object.values(cursors), onChange };
}
