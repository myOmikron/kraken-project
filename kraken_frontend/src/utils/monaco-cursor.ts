import { editor } from "monaco-editor";
import React from "react";
import { createPortal } from "react-dom";
import TrackedRangeStickiness = editor.TrackedRangeStickiness;

/** monaco decoration placed at others' cursor positions */
const CURSOR_DECO: editor.IModelDecorationOptions = {
    className: "cursor-deco",
    stickiness: TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
};

export default class Cursor {
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

    constructor(
        editorInstance: editor.IStandaloneCodeEditor | null,
        line: number,
        column: number,
        active: boolean = true
    ) {
        this.line = line;
        this.column = column;

        this.id = `cursor-${Cursor.nextId++}`;
        this.active = active;

        this.node = document.createElement("div");
        this.node.classList.add("cursor-root");
        this.widget = {
            getId: () => this.id,
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
        if (this.active) {
            if (this.decoration !== null) this.decoration.set([this.getDeco()]);
            if (this.editorInstance !== null) this.editorInstance.layoutContentWidget(this.widget);
        }
    }

    /** Updates whether the cursor is currently visible or not */
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

    /** Renders a React node at the cursor */
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

    /** Sets a new editor **without** clearing a previously set one */
    private setEditor(editorInstance: editor.IStandaloneCodeEditor) {
        this.editorInstance = editorInstance;
        this.decoration = editorInstance.createDecorationsCollection(this.active ? [this.getDeco()] : []);
        if (this.active) editorInstance.addContentWidget(this.widget);
    }

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
