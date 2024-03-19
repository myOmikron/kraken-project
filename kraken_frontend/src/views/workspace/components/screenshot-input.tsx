import React, { ChangeEvent, DragEvent, forwardRef } from "react";
import Popup from "reactjs-popup";
import "../../../styling/screenshot-input.css";
import CloseIcon from "../../../svg/close";
import { toast } from "react-toastify";
import { WORKSPACE_CONTEXT } from "../workspace";

export type ScreenshotInputProps = {
    /** The image as either a {@link File} or an uuid for an uploaded image*/
    screenshot: File | string | undefined;

    /** Callback invoked when the user changed the selected image */
    onChange: (newFile: File | undefined) => void;

    /** Toggles the component between compact and bigger */
    shortText?: boolean;
} & Omit<React.HTMLProps<HTMLDivElement>, "onChange">;

export const ScreenshotInput = forwardRef<HTMLDivElement, ScreenshotInputProps>(
    ({ screenshot: image, onChange, shortText, ...props }, ref) => {
        const {
            workspace: { uuid: workspace },
        } = React.useContext(WORKSPACE_CONTEXT);

        const [drag, setDrag] = React.useState<boolean>(false);
        const [popup, setPopup] = React.useState<boolean>(false);

        const [blobURL, setBlobURL] = React.useState("");
        React.useEffect(() => {
            if (image !== undefined && typeof image !== "string") {
                const url = URL.createObjectURL(image);
                setBlobURL(url);
                return () => URL.revokeObjectURL(url);
            }
        }, [image]);
        const fullImageUrl =
            typeof image === "object" ? blobURL : image && `/api/v1/workspace/${workspace}/files/${image}`;
        const thumbnailUrl =
            typeof image === "object" ? blobURL : image && `/api/v1/workspace/${workspace}/files/${image}/thumbnail`;

        const inputRef = React.useRef<HTMLInputElement>(null);

        const dropHandler = (e: DragEvent<HTMLInputElement>) => {
            console.log(e);
            e.preventDefault();
            if (inputRef.current) {
                inputRef.current.files = e.dataTransfer.files;
                // updateImage();
            }
        };

        return (
            <div {...props} className={`screenshot-input ${props.className}`} ref={ref}>
                {image && (
                    <button
                        title={"Remove screenshot"}
                        className="remove"
                        onClick={() => {
                            onChange(undefined);
                            if (inputRef.current) {
                                // clear file
                                inputRef.current.value = null as any;
                            }
                        }}
                    >
                        <CloseIcon />
                    </button>
                )}
                <div
                    className="content"
                    onDrop={dropHandler}
                    onDragOver={(e) => {
                        if (!drag) {
                            setDrag(true);
                        }
                        e.preventDefault();
                    }}
                    onDragEnter={(e) => {
                        e.preventDefault();
                    }}
                    onDragLeave={() => setDrag(false)}
                >
                    {thumbnailUrl ? (
                        <div
                            className="image-preview"
                            onClick={() => {
                                setPopup(true);
                            }}
                        >
                            <img src={thumbnailUrl} alt={"Screenshot Thumbnail"} />
                        </div>
                    ) : (
                        <>
                            {props.children}
                            {drag ? (
                                <span>{shortText ? "Drop image" : "Drop image here"}</span>
                            ) : (
                                <span>{shortText ? "Upload image" : "Drag your image here or click in this area"}</span>
                            )}
                        </>
                    )}

                    <input
                        ref={inputRef}
                        type="file"
                        onChange={(event) => {
                            const image = event.target.files?.[0];

                            if (!image) {
                                return toast.error("No file selected");
                            }

                            if (!image.type.includes("image")) {
                                return toast.error("File must be .png .jpg .jpeg");
                            }

                            if (image.size > 100 * 1024 * 1024) {
                                return toast.error("File too large");
                            }

                            onChange(image);
                            setDrag(false);
                        }}
                        accept={".jpg, .jpeg, .png"}
                    />
                </div>
                <Popup className="screenshot-popup" nested modal open={popup} onClose={() => setPopup(false)}>
                    <div className="screenshot-input-popup" onClick={() => setPopup(false)}>
                        <img src={fullImageUrl} alt={"Screenshot"} />
                    </div>
                </Popup>
            </div>
        );
    },
);
