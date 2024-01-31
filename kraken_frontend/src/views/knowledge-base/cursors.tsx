import { editor } from "monaco-editor";
import React from "react";
import { createPortal } from "react-dom";
import TrackedRangeStickiness = editor.TrackedRangeStickiness;

/** monaco decoration placed at others' cursor positions */
const CURSOR_DECO: editor.IModelDecorationOptions = {
    className: "cursor-deco",
    stickiness: TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
};

/**
 * Stores cursors in internal state and exposes simple `insert` and `remove` functions to update them
 *
 * This hook is intended to be used in combination with {@link Cursors `<CursorLabels />`}
 * which adds labels (arbitrary React nodes) to the cursors.
 */
export function useCursors<U extends { uuid: string }>(editorInstance: null | editor.IStandaloneCodeEditor) {
    const [cursors, setCursors] = React.useState<Record<string, Cursor<U>>>({});

    // Updates cursors which have been created before the editor was available
    React.useEffect(() => {
        for (const uuid in cursors) {
            if (Object.hasOwn(cursors, uuid)) {
                const cursor = cursors[uuid];
                cursor.updateEditor(editorInstance);
            }
        }
    }, [editorInstance]);

    const insert = React.useCallback(
        (user: U, line: number, column: number) => {
            setCursors(({ ...cursors }) => {
                if (user.uuid in cursors) {
                    cursors[user.uuid].updatePosition(line, column);
                } else {
                    cursors[user.uuid] = new Cursor(editorInstance, user, line, column);
                }
                return cursors;
            });
        },
        [editorInstance, setCursors],
    );

    const remove = React.useCallback(
        (user: U) => {
            setCursors(({ [user.uuid]: oldCursor, ...cursors }) => {
                if (oldCursor !== undefined) oldCursor.delete();
                return cursors;
            });
        },
        [setCursors],
    );

    return {
        cursors,
        insert,
        remove,
    };
}

/** Properties for {@link CursorLabels `<CursorLabels />`} */
type CursorLabelsProps<U extends { uuid: string }> = {
    cursors: Record<string, Cursor<U>>;
    children: (user: U) => React.ReactNode;
};

/**
 * Adds labels to a set of cursors
 *
 * The cursors are intended to be provided by the {@link useCursors `useCursors`} hook.
 *
 * This attaches to the {@link Cursor `Cursor`}'s internal dom node using {@link createPortal `createPortal`}.
 *
 * Therefore, only one instance of `<CursorLabels />` can render to a {@link Cursor `Cursor`} at a time.
 */
export function CursorLabels<U extends { uuid: string }>(props: CursorLabelsProps<U>) {
    return <>{Object.values(props.cursors).map((cursor) => cursor.createPortal(props.children(cursor.user)))}</>;
}

class Cursor<U extends { uuid: string }> {
    user: U;
    line: number;
    column: number;

    private editorInstance: editor.IStandaloneCodeEditor | null = null;
    private decoration: editor.IEditorDecorationsCollection | null = null;

    private readonly node: HTMLElement;
    private readonly widget: editor.IContentWidget;

    constructor(editorInstance: editor.IStandaloneCodeEditor | null, user: U, line: number, column: number) {
        this.line = line;
        this.column = column;
        this.user = user;

        this.node = document.createElement("div");
        this.node.classList.add("cursor-root");
        this.widget = {
            getId: () => `cursor-${user.uuid}`,
            getPosition: () => ({
                position: { lineNumber: this.line, column: this.column },
                preference: [
                    editor.ContentWidgetPositionPreference.ABOVE,
                    editor.ContentWidgetPositionPreference.BELOW,
                    editor.ContentWidgetPositionPreference.EXACT,
                ],
            }),
            getDomNode: () => this.node,
        };

        if (editorInstance !== null) this.setEditor(editorInstance);
    }

    /**
     * Updates the editor this cursor is shown in
     *
     * I.e. The cursor is removed from its old editor and attached to the new one.
     */
    updateEditor(editorInstance: editor.IStandaloneCodeEditor | null) {
        this.removeEditor();
        if (editorInstance !== null) this.setEditor(editorInstance);
    }

    /** Updates the cursor's position */
    updatePosition(line: number, column: number) {
        this.line = line;
        this.column = column;
        if (this.decoration !== null)
            this.decoration.set([
                {
                    range: {
                        startLineNumber: this.line,
                        endLineNumber: this.line,
                        startColumn: this.column,
                        endColumn: this.column,
                    },
                    options: CURSOR_DECO,
                },
            ]);
        if (this.editorInstance !== null) this.editorInstance.layoutContentWidget(this.widget);
    }

    /** Renders a React node at the cursor */
    createPortal(children: React.ReactNode) {
        return createPortal(children, this.node);
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
        if (this.editorInstance !== null) {
            this.editorInstance.removeContentWidget(this.widget);
            this.editorInstance = null;
        }
        if (this.decoration !== null) {
            this.decoration.clear();
            this.decoration = null;
        }
    }

    /** Sets a new editor **without** clearing a previously set one */
    private setEditor(editorInstance: editor.IStandaloneCodeEditor) {
        this.editorInstance = editorInstance;
        this.decoration = editorInstance.createDecorationsCollection([
            {
                range: {
                    startLineNumber: this.line,
                    endLineNumber: this.line,
                    startColumn: this.column,
                    endColumn: this.column,
                },
                options: CURSOR_DECO,
            },
        ]);
        editorInstance.addContentWidget(this.widget);
    }
}
