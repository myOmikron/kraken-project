import React from "react";
import { loader, Monaco } from "@monaco-editor/react";
import { editor, IDisposable, Uri } from "monaco-editor";
import Loading from "../components/loading";
import { MONACO, MONACO_PROMISE } from "./monaco";
import ITextModel = editor.ITextModel;
import { EditorTarget } from "../api/generated";
import { ListenerHandle } from "./event-emitter";
import WS from "../api/websocket";
import { ObjectFns } from "./helper";
import USER_CONTEXT from "../context/user";

/**
 * Shitty switch to en- and disable logging
 *
 * Set to `console` to enable or `null` to disable
 */
const CONSOLE: Console | null = console;

/** {@link useModel `useModel`}' arguments */
export type UseModelArgs = {
    language?: string;
    syncTarget?: EditorTarget;
    value?: string;
};

/** {@link useModel `useModel`}'s return value */
export type UseModelReturn = [
    string,
    (text: string, syncTarget: EditorTarget | undefined) => void,
    editor.ITextModel | null,
];

/**
 * TODO: more docs
 *
 * The returned tuple tries to mimic {@link React.useState `React.useState`}'s return type.
 *
 * The first two elements are the current state and a setter to update it.
 *
 * (Difference from `React.useState`: the setter only accepts a new value, it doesn't accept an update function!)
 *
 * The third element is monaco's representation of the state (aka a {@link ITextModel model})
 * which can be passed to a {@link ModelEditor `<ModelEditor />`}.
 */
export function useModel(initialArgs: UseModelArgs): UseModelReturn {
    const {
        user: { uuid: user },
    } = React.useContext(USER_CONTEXT);
    const { current: controller } = React.useRef(new ModelController(initialArgs, user));

    controller.trigger = useTrigger();
    controller.onRender();

    React.useEffect(() => {
        controller.onMount();
        return () => {
            controller.onUnmount();
        };
    }, []);

    return [controller.value, controller.setValue.bind(controller), controller.model];
}

/** {@link useModelStore `useModelStore`}'s return value */
export type UseModelStoreReturn = {
    models: Record<
        string,
        {
            value: string;
            model: ITextModel | null;
        }
    >;
    addModel: (key: string, args: { language: string; value: string; syncTarget: EditorTarget }) => void;
    removeModel: (key: string) => void;
    removeAll: () => void;
};

export function useModelStore(): UseModelStoreReturn {
    const {
        user: { uuid: user },
    } = React.useContext(USER_CONTEXT);
    const [controllers, setControllers] = React.useState<Record<string, ModelController>>({});

    const trigger = useTrigger();
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
        addModel: (key: string, args: UseModelArgs) => {
            setControllers(({ [key]: controller, ...rest }) => {
                if (controller) controller.onUnmount();
                needsMount.current.push(key);
                return { [key]: new ModelController(args, user), ...rest };
            });
        },
        removeModel: (key: string) => {
            setControllers(({ [key]: controller, ...rest }) => {
                if (controller) controller.onUnmount();
                return rest;
            });
        },
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
 * Hacky helper to construct the `trigger` passed to `ModelController`
 *
 * This hook simply returns a function which, when called, will trigger a rerender.
 */
function useTrigger() {
    const [_, setDummy] = React.useState({});
    return () => setDummy({});
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
export class ModelController {
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
    websocketListener: ListenerHandle<any> | null = null;

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

            CONSOLE?.group("onFirstRender");

            // Try to load the model on initial render.
            if (MONACO !== null) {
                CONSOLE?.debug("Monaco available already");
                this.setupModel(MONACO);
            } else {
                CONSOLE?.debug("Monaco not available yet");
            }

            CONSOLE?.groupEnd();
        }
    }

    /** Called once **after** mounting */
    onMount() {
        CONSOLE?.group("onMount");

        // If we couldn't load the model on initial render,
        // enqueue it.
        if (this.model === null) {
            CONSOLE?.debug("Waiting for monaco to be available");
            MONACO_PROMISE.then((monaco) => {
                this.setupModel(monaco);
                this.trigger();
            });
        }

        CONSOLE?.groupEnd();
    }

    /** Called once **after** unmounting */
    onUnmount() {
        CONSOLE?.group("onUnmount");

        if (this.websocketListener !== null) {
            CONSOLE?.debug("Removing WS listener");
            WS.removeEventListener(this.websocketListener);
            this.websocketListener = null;
        } else {
            CONSOLE?.debug("WS listener was already removed");
        }
        if (this.monacoListener !== null) {
            CONSOLE?.debug("Removing monaco listener");
            this.monacoListener.dispose();
            this.monacoListener = null;
        } else {
            CONSOLE?.debug("Monaco listener was already removed");
        }
        if (this.model !== null) {
            CONSOLE?.debug("Disposing monaco model");
            this.model.dispose();
            this.model = null;
        } else {
            CONSOLE?.debug("Monaco model was already disposed");
        }

        CONSOLE?.groupEnd();
    }

    /** Called once as soon as monaco is available */
    setupModel(monaco: Monaco) {
        CONSOLE?.group("setupModel");

        CONSOLE?.debug({ value: this.value, language: this.language });

        const model = monaco.editor.createModel(this.value, this.language);
        this.model = model;
        this.monacoListener = model.onDidChangeContent((event) => {
            CONSOLE?.group("model.onDidChangeContent");

            CONSOLE?.debug({ source: this.updateSource });

            this.value = model.getValue();
            this.trigger();

            if (this.updateSource === "monaco" && this.syncTarget !== undefined) {
                CONSOLE?.debug("Sending changes over WS:", { target: this.syncTarget });

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

            CONSOLE?.groupEnd();
        });

        CONSOLE?.groupEnd();
    }

    /** Called once in `constructor` */
    setupWS() {
        this.websocketListener = WS.addEventListener("message.EditorChangedContent", (event) => {
            if (this.syncTarget === undefined) return;

            // Annoying workaround for the bad code generated from our openapi.json
            // @ts-ignore
            const [key] = ObjectFns.keys(this.syncTarget);
            // @ts-ignore
            const equalsTargets = ObjectFns.deepEquals(event.target[key], this.syncTarget[key]);

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

    /** The `setValue` function returned by the `useModel` hook */
    setValue(newValue: string, syncTarget: EditorTarget | undefined) {
        CONSOLE?.group("setValue");
        CONSOLE?.debug({ hasModel: this.model !== null, oldValue: this.value, newValue });

        this.value = newValue;
        this.syncTarget = syncTarget;
        if (this.model) {
            this.updateSource = "react";
            this.model.setValue(newValue);
            this.updateSource = "monaco";
        }

        CONSOLE?.groupEnd();
    }
}
