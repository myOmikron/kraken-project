import { editor, IDisposable } from "monaco-editor";
import React from "react";
import { createPortal } from "react-dom";
import { EditorTarget, SimpleUser } from "../api/generated";
import WS from "../api/websocket";
import USER_CONTEXT from "../context/user";
import { useStableObj } from "./helper";
import TrackedRangeStickiness = editor.TrackedRangeStickiness;

/**
 * Arguments to the {@link useSyncedCursors} hook
 */
export type UseSyncedCursorsArgs<CustomData extends {} = true> = {
    /**
     * The {@link EditorTarget} to use when sending updates to the server
     */
    target: EditorTarget;

    /**
     * Filter an incoming cursor update for the right target and extract it
     *
     * Return `undefined` if the update should be ignored (i.e. you may omit the return statement)
     */
    receiveCursor: (target: EditorTarget) => CustomData | undefined;

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
    isCursorHidden?: (cursor: SimpleUser & CustomData) => boolean;

    /**
     * Optional flag to include the own cursor
     *
     * Normally the user's own cursor won't be included.
     */
    includeOwnCursor?: boolean;
};

/**
 * Synchronizes cursors of different users
 *
 * This hook can only operate on a single editor instance at a time.
 * This instance can be switched any time moving all cursors from the old one to the new one.
 *
 * @param args various configuration options as well as the required `target` and `receiveCursor`.
 *
 * @returns
 *     - `setEditor` to connect this hook with your monaco editor
 *     - `cursors`: a list of the cursors
 */
export function useSyncedCursors<CustomData extends {} = true>(args: UseSyncedCursorsArgs<CustomData>) {
    const {
        target,
        receiveCursor,
        deleteCursors = [],
        hideCursors = [],
        isCursorHidden,
        includeOwnCursor = false,
    } = args;
    const { user } = React.useContext(USER_CONTEXT);
    const [editorInstance, setEditor] = React.useState<editor.IStandaloneCodeEditor | null>(null);
    const stableArgs = useStableObj({
        target,
        receiveCursor,
        isCursorHidden,
        includeOwnCursor,
        user: user.uuid,
    });

    /** The stored cursors */
    type Cursors = Record<
        string,
        {
            /** Some application state associated with the cursor */
            data: SimpleUser & CustomData;
            /** The cursor's monaco state*/
            cursor: Cursor;
        }
    >;
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
        let disposable: IDisposable = {
            // eslint-disable-next-line jsdoc/require-jsdoc
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

    return { cursors: Object.values(cursors), setEditor };
}

/** monaco decoration placed at others' cursor positions */
const CURSOR_DECO: editor.IModelDecorationOptions = {
    className: "cursor-deco",
    stickiness: TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
};

/**
 * A cursor attached to a single monaco editor which can render a React node at its position
 *
 * The cursor is realised as an editor decoration of width zero and css class `"cursor-deco"`.
 * It also uses monaco's widgets to place a `<div class="cursor-root" />` at the cursor's position
 * which is computed by monaco.
 */
export class Cursor {
    private static nextId = 0;

    /** The cursor's line number (1-indexed!) */
    line: number;
    /** The cursor's column number (1-indexed!) */
    column: number;

    /** An id used by monaco and React */
    readonly id: `cursor-${number}`;

    /** Flag whether the cursor is "active" i.e. displayed */
    private active: boolean = false;

    /** The current editor this cursor is attached to */
    private editorInstance: editor.IStandaloneCodeEditor | null = null;
    /**
     * A decoration collection containing a single decoration
     *
     * `this.decoration === null` <=> `this.editorInstance === null`
     */
    private decoration: editor.IEditorDecorationsCollection | null = null;

    /** A `<div class="cursor-root" />` which will be positioned at the cursor by monaco */
    private readonly node: HTMLElement;
    /** The config for positioning the `node` */
    private readonly widget: editor.IContentWidget;

    /**
     * Constructs a new cursor
     *
     * @param editorInstance the initial editor this cursor should be attached to
     * @param line the cursor's initial line number (1-indexed!)
     * @param column the cursor's initial column number (1-indexed!)
     * @param active should the cursors start as visible?
     */
    constructor(
        editorInstance: editor.IStandaloneCodeEditor | null,
        line: number,
        column: number,
        active: boolean = true,
    ) {
        this.line = line;
        this.column = column;

        this.id = `cursor-${Cursor.nextId++}`;
        this.active = active;

        this.node = document.createElement("div");
        this.node.classList.add("cursor-root");
        this.widget = {
            // eslint-disable-next-line jsdoc/require-jsdoc
            getId: () => this.id,
            // eslint-disable-next-line jsdoc/require-jsdoc
            getPosition: () => ({
                position: { lineNumber: this.line, column: this.column },
                preference: [
                    editor.ContentWidgetPositionPreference.ABOVE,
                    editor.ContentWidgetPositionPreference.BELOW,
                    editor.ContentWidgetPositionPreference.EXACT,
                ],
            }),
            // eslint-disable-next-line jsdoc/require-jsdoc
            getDomNode: () => this.node,
        };

        if (editorInstance !== null) this.setEditor(editorInstance);
    }

    /**
     * Updates the editor this cursor is shown in
     *
     * I.e. The cursor is removed from its old editor and attached to the new one.
     *
     * @param editorInstance the cursor's new editor instance
     */
    updateEditor(editorInstance: editor.IStandaloneCodeEditor | null) {
        this.removeEditor();
        if (editorInstance !== null) this.setEditor(editorInstance);
    }

    /**
     * Updates the cursor's position
     *
     * @param line the cursor's new line number (1-indexed!)
     * @param column the cursor's new column number (1-indexed!)
     */
    updatePosition(line: number, column: number) {
        this.line = line;
        this.column = column;
        if (this.active) {
            if (this.decoration !== null) this.decoration.set([this.getDeco()]);
            if (this.editorInstance !== null) this.editorInstance.layoutContentWidget(this.widget);
        }
    }

    /**
     * Updates whether the cursor is currently visible or not
     *
     * @param active should the cursor be currently visible?
     */
    updateActive(active: boolean) {
        // Disable
        if (this.active && !active) {
            if (this.decoration !== null) this.decoration.clear();
            if (this.editorInstance !== null) this.editorInstance.removeContentWidget(this.widget);
        }
        // Enable
        else if (!this.active && active) {
            if (this.decoration !== null) this.decoration.set([this.getDeco()]);
            if (this.editorInstance !== null) this.editorInstance.addContentWidget(this.widget);
        }
        this.active = active;
    }

    /**
     * Renders a React node at the cursor
     *
     * This method takes the `active` state already into account,
     * so a caller can simply map all of his cursors without filtering
     * for the `active` state.
     *
     * This boils down to a simple {@link createPortal `ReactDOM.createPortal`},
     * so you have to include this methods return value into the caller's JSX.
     *
     * @param children the React node to render at the cursor's position
     * @returns a React portal rendering `children` at the position of the cursor
     */
    render(children: React.ReactNode) {
        if (this.active) return createPortal(children, this.node, this.id);
        else return null;
    }

    /**
     * Deletes the cursor
     *
     * Don't continue to use the cursor after calling this method!
     */
    delete() {
        this.removeEditor();
        this.node.remove();
    }

    /** Removes the cursor from its current editor (if any) */
    private removeEditor() {
        if (this.active) {
            if (this.editorInstance !== null) {
                this.editorInstance.removeContentWidget(this.widget);
            }
            if (this.decoration !== null) {
                this.decoration.clear();
            }
        }
        this.editorInstance = null;
        this.decoration = null;
    }

    /**
     * Sets a new editor **without** clearing a previously set one
     *
     * @param editorInstance the editor instance to set for the cursor
     */
    private setEditor(editorInstance: editor.IStandaloneCodeEditor) {
        this.editorInstance = editorInstance;
        this.decoration = editorInstance.createDecorationsCollection(this.active ? [this.getDeco()] : []);
        if (this.active) editorInstance.addContentWidget(this.widget);
    }

    /**
     * Construct the decoration monaco should put into the editor for representing the cursor
     *
     * @returns the decoration representing the cursor
     */
    private getDeco(): editor.IModelDeltaDecoration {
        return {
            range: {
                startLineNumber: this.line,
                endLineNumber: this.line,
                startColumn: this.column,
                endColumn: this.column,
            },
            options: CURSOR_DECO,
        };
    }
}
