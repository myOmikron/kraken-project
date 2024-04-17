import { Monaco } from "@monaco-editor/react";
import { editor, IDisposable } from "monaco-editor";
import React from "react";
import { EditorTarget } from "../api/generated";
import WS from "../api/websocket";
import USER_CONTEXT from "../context/user";
import CONSOLE from "./console";
import { ListenerHandle } from "./event-emitter";
import { ObjectFns } from "./helper";
import { MONACO, MONACO_PROMISE } from "./monaco";
import { useTriggerUpdate } from "./trigger-hook";
import ITextModel = editor.ITextModel;

/** {@link useModel `useModel`}' arguments */
export type UseModelArgs = {
    /** The language the model should use */
    language?: string;
    /** The {@link EditorTarget} to synchronize changes with */
    syncTarget?: EditorTarget;
    /** The model's value */
    value?: string;
};

/** {@link useModel `useModel`}'s return value */
export type UseModelReturn = [
    string,
    (text: string, syncTarget: EditorTarget | undefined) => void,
    editor.ITextModel | null,
];

/**
 * Returns a stateful string and a function for updating it which are kept in sync with a monaco model.
 *
 * This hook bridges the gap between React's and monaco's state management.
 *
 * ## Return
 *
 * The returned tuple tries to mimic {@link React.useState `React.useState`}'s return type.
 *
 * The first two elements are the current state and a setter to update it.
 *
 * (Difference from `React.useState`: the setter only accepts a new value, it doesn't accept an update function!)
 *
 * The third element is monaco's representation of the state (aka a {@link ITextModel model})
 * which can be passed to a {@link ModelEditor `<ModelEditor />`}.
 *
 * ## Sync
 *
 * This hook also manages the synchronization of your model over the websocket.
 *
 * Simply provide the `syncTarget` in the `initialArgs`.
 *
 * If you want to change the `syncTarget` (for example because you're edit view switches to another uuid),
 * you can pass a new `syncTarget` as second argument to the update function you use to switch the model's value.
 *
 * @param initialArgs the **initial** parameters the model should be created with.
 *     - changes to those values in re-renders will be ignored!
 *
 * @returns tuple of:
 *     1. the model's current value
 *     2. a function to update the model's current value (and change its sync target)
 *     3. the model itself
 */
export function useModel(initialArgs: UseModelArgs): UseModelReturn {
    const {
        user: { uuid: user },
    } = React.useContext(USER_CONTEXT);
    const { current: controller } = React.useRef(new ModelController(initialArgs, user));

    controller.trigger = useTriggerUpdate();
    controller.onRender();

    React.useEffect(() => {
        controller.onMount();
        return () => {
            controller.onUnmount();
        };
    }, []);

    return [controller.value, controller.setValue, controller.model];
}

/** A model store as returned by {@link useModelStore `useModelStore`} */
export type ModelStore = {
    /** The store's current models and their values */
    models: Record<
        string,
        {
            /** The model's value */
            value: string;
            /** The model */
            model: ITextModel | null;
        }
    >;
    /** Adds a new model to the store (overwriting any existing one) */
    addModel: (key: string, args: UseModelArgs) => void;
    /** Removes a single model from the store */
    removeModel: (key: string) => void;
    /** Removes all models from the store */
    removeAll: () => void;
};

/**
 * A more dynamic version of {@link useModel `useModel`}.
 *
 * Due to the ["Rules of Hooks"](https://react.dev/reference/rules/rules-of-hooks) you can only get
 * a predefined number of models using `useModel`.
 *
 * This hook creates a "store" which can hold a completely dynamic number of models.
 *
 * Models are stored in an {@link ModelStore#models object} and identified by arbitrary strings provided by the caller.
 *
 * @returns a model store i.e. a set of models and a few functions to modify it
 */
export function useModelStore(): ModelStore {
    const {
        user: { uuid: user },
    } = React.useContext(USER_CONTEXT);
    const [controllers, setControllers] = React.useState<Record<string, ModelController>>({});

    const trigger = useTriggerUpdate();
    for (const controller of Object.values(controllers)) {
        controller.trigger = trigger;
        controller.onRender();
    }

    const needsMount = React.useRef(Array<string>());
    React.useEffect(() => {
        for (const key of needsMount.current) {
            if (key in controllers) controllers[key].onMount();
        }
        needsMount.current = [];
    }, [controllers]);

    React.useEffect(
        () => () => {
            for (const controller of Object.values(controllers)) {
                controller.onUnmount();
            }
        },
        [],
    );

    return {
        models: Object.fromEntries(
            Object.entries(controllers).map(([key, controller]) => [
                key,
                {
                    value: controller.value,
                    model: controller.model,
                },
            ]),
        ),
        // eslint-disable-next-line jsdoc/require-jsdoc
        addModel: (key: string, args: UseModelArgs) => {
            setControllers(({ [key]: controller, ...rest }) => {
                if (controller) controller.onUnmount();
                needsMount.current.push(key);
                return { [key]: new ModelController(args, user), ...rest };
            });
        },
        // eslint-disable-next-line jsdoc/require-jsdoc
        removeModel: (key: string) => {
            setControllers(({ [key]: controller, ...rest }) => {
                if (controller) controller.onUnmount();
                return rest;
            });
        },
        // eslint-disable-next-line jsdoc/require-jsdoc
        removeAll: () => {
            setControllers((controllers) => {
                for (const controller of Object.values(controllers)) {
                    controller.onUnmount();
                }
                return {};
            });
        },
    };
}

/**
 * Class bridging monaco state with react state and optionally synchronizing it over the websocket.
 *
 * It stores and controls monaco state internally and interacts with react state through the `useModel` hook.
 * When setting the value either in the constructor or using `setValue` an {@link EditorTarget} can be passed,
 * indicating where the value came from to keep it in sync.
 *
 * **Don't construct an instance yourself, use {@link useModel `useModel`}!**
 */
class ModelController {
    /**
     * Language used by the model
     *
     * This string is used once upon model creation and ignored after that.
     */
    readonly language: string | undefined;
    /**
     * The model's value
     *
     * This string has to be kept in sync with {@link ITextModel.getValue `model.getValue()`}.
     * `this.trigger` has to be invoked after changing this string to notify react about the change.
     */
    value: string;
    /** Function to trigger a React update */
    trigger = () => {};

    /**
     * Identifier to use when communicating changes over the websocket.
     *
     * Set `undefined` if the model should not be synced.
     */
    syncTarget: EditorTarget | undefined;
    /** The user's uuid used to filter incoming websocket messages */
    user: string;

    /** The controlled monaco model */
    model: ITextModel | null = null;

    /**
     * Listener for changes to the monaco model
     *
     * This listener will be created alongside the model and exist until `onUnmount`.
     */
    monacoListener: IDisposable | null = null;

    /**
     * Listener for changes coming from the websocket
     *
     * This listener will be created in the `constructor` and exist until `onUnmount`.
     */
    websocketListener: ListenerHandle<"message.EditorChangedContent"> | null = null;

    /** Is `onRender` invoked for the first time? */
    firstRender = true;

    /**
     * Enum indicating why the listener for model changes triggered
     *
     * When modifying the model's value from code, change this value like a guard:
     * ```js
     * this.updateSource = "something";
     * this.model.update(somehow);
     * this.updateSource = "monaco";
     * ```
     */
    updateSource: "websocket" | "monaco" | "react" = "monaco";

    // eslint-disable-next-line jsdoc/require-jsdoc
    constructor(args: UseModelArgs, user: string) {
        this.language = args.language;
        this.syncTarget = args.syncTarget;
        this.value = args.value ?? "";

        this.user = user;
        this.setupWS();
    }

    /** Called each render */
    onRender() {
        if (this.firstRender) {
            this.firstRender = false;

            CONSOLE.group("onFirstRender");

            // Try to load the model on initial render.
            if (MONACO !== null) {
                CONSOLE.debug("Monaco available already");
                this.setupModel(MONACO);
            } else {
                CONSOLE.debug("Monaco not available yet");
            }

            CONSOLE.groupEnd();
        }
    }

    /** Called once **after** mounting */
    onMount() {
        CONSOLE.group("onMount");

        // If we couldn't load the model on initial render,
        // enqueue it.
        if (this.model === null) {
            CONSOLE.debug("Waiting for monaco to be available");
            MONACO_PROMISE.then((monaco) => {
                this.setupModel(monaco);
                this.trigger();
            });
        }

        CONSOLE.groupEnd();
    }

    /** Called once **after** unmounting */
    onUnmount() {
        CONSOLE.group("onUnmount");

        if (this.websocketListener !== null) {
            CONSOLE.debug("Removing WS listener");
            WS.removeEventListener(this.websocketListener);
            this.websocketListener = null;
        } else {
            CONSOLE.debug("WS listener was already removed");
        }
        if (this.monacoListener !== null) {
            CONSOLE.debug("Removing monaco listener");
            this.monacoListener.dispose();
            this.monacoListener = null;
        } else {
            CONSOLE.debug("Monaco listener was already removed");
        }
        if (this.model !== null) {
            CONSOLE.debug("Disposing monaco model");
            this.model.dispose();
            this.model = null;
        } else {
            CONSOLE.debug("Monaco model was already disposed");
        }

        CONSOLE.groupEnd();
    }

    // eslint-disable-next-line jsdoc/require-param
    /** Called once as soon as monaco is available */
    setupModel(monaco: Monaco) {
        CONSOLE.group("setupModel");

        CONSOLE.debug({ value: this.value, language: this.language });

        const model = monaco.editor.createModel(this.value, this.language);
        this.model = model;
        this.monacoListener = model.onDidChangeContent((event) => {
            CONSOLE.group("model.onDidChangeContent");

            CONSOLE.debug({ source: this.updateSource });

            this.value = model.getValue();
            this.trigger();

            if (this.updateSource === "monaco" && this.syncTarget !== undefined) {
                CONSOLE.debug("Sending changes over WS:", { target: this.syncTarget });

                // Send the changes to the websocket
                for (const change of event.changes) {
                    const {
                        text,
                        range: { startColumn, startLineNumber, endLineNumber, endColumn },
                    } = change;
                    WS.send({
                        type: "EditorChangedContent",
                        target: this.syncTarget,
                        change: {
                            text,
                            startColumn,
                            endColumn,
                            startLine: startLineNumber,
                            endLine: endLineNumber,
                        },
                    });
                }
            }

            CONSOLE.groupEnd();
        });

        CONSOLE.groupEnd();
    }

    /** Called once in `constructor` */
    setupWS() {
        this.websocketListener = WS.addEventListener("message.EditorChangedContent", (event) => {
            if (this.syncTarget === undefined) return;

            // Annoying workaround for the bad code generated from our openapi.json
            const [key] = ObjectFns.keys(this.syncTarget);
            // @ts-ignore: implicit any is fine, since deepEquals handles unknown
            const equalsTargets = ObjectFns.deepEquals(event.target[key] as unknown, this.syncTarget[key] as unknown);

            if (equalsTargets) {
                if (event.user.uuid === this.user) return; // TODO: might need more consideration

                const { text, startColumn, startLine, endColumn, endLine } = event.change;

                this.updateSource = "websocket";
                this.model?.applyEdits([
                    { range: { startColumn, endColumn, startLineNumber: startLine, endLineNumber: endLine }, text },
                ]);
                this.updateSource = "monaco";
            }
        });
    }

    // eslint-disable-next-line jsdoc/require-param
    /**
     * The `setValue` function returned by the `useModel` hook
     *
     * This function uses arrow notation, because it needs to be bound to `this`,
     * for `useModel` to return it.
     */
    setValue = (newValue: string, syncTarget: EditorTarget | undefined) => {
        CONSOLE.group("setValue");
        CONSOLE.debug({ hasModel: this.model !== null, oldValue: this.value, newValue });

        this.value = newValue;
        this.syncTarget = syncTarget;
        if (this.model) {
            this.updateSource = "react";
            this.model.setValue(newValue);
            this.updateSource = "monaco";
        }

        CONSOLE.groupEnd();
    };
}
