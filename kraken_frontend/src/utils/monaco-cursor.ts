import { editor } from "monaco-editor";
import React from "react";
import { createPortal } from "react-dom";
import TrackedRangeStickiness = editor.TrackedRangeStickiness;

/** monaco decoration placed at others' cursor positions */
const CURSOR_DECO: editor.IModelDecorationOptions = {
    className: "cursor-deco",
    stickiness: TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
};

export default class Cursor<U extends { uuid: string }> {
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
    render(children: React.ReactNode) {
        return createPortal(children, this.node, this.user.uuid);
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
