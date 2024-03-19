import React, { ChangeEvent, DragEvent, forwardRef } from "react";
import Popup from "reactjs-popup";
import "../../../styling/screenshot-input.css";
import CloseIcon from "../../../svg/close";
import { toast } from "react-toastify";

export type Screenshot = {
    file?: File;
    dataURL: string;
};

export type ScreenshotInputProps = {
    screenshot: Screenshot | undefined;
    onChange: React.Dispatch<Screenshot | undefined>;
    shortText?: boolean;
} & Omit<React.HTMLProps<HTMLDivElement>, "onChange">;

export const ScreenshotInput = forwardRef<HTMLDivElement, ScreenshotInputProps>(
    ({ screenshot, onChange, shortText, ...props }, ref) => {
        const [drag, setDrag] = React.useState<boolean>(false);
        const [popup, setPopup] = React.useState<boolean>(false);

        const inputRef = React.useRef<HTMLInputElement>(null);

        const updateImage = () => {
            const fileUploadInput = inputRef.current;

            if (!fileUploadInput) {
                return toast.error("No file input element found");
            }

            // @ts-ignore
            const image = fileUploadInput.files[0];

            if (!image) {
                return toast.error("No file selected");
            }

            if (!image.type.includes("image")) {
                return toast.error("File must be .png .jpg .jpeg");
            }

            if (image.size > 100 * 1024 * 1024) {
                return toast.error("File too large");
            }

            const fileReader = new FileReader();
            fileReader.readAsDataURL(image);

            fileReader.onload = (e) => {
                const result = e.target?.result as string;
                if (result) {
                    onChange({
                        dataURL: result,
                        file: image,
                    });
                }
            };
            setDrag(false);
        };
        const dropHandler = (e: DragEvent<HTMLInputElement>) => {
            console.log(e);
            e.preventDefault();
            if (inputRef.current) {
                inputRef.current.files = e.dataTransfer.files;
                // updateImage();
            }
        };

        const imageHandler = (e: ChangeEvent<HTMLInputElement>) => {
            updateImage();
        };

        return (
            <div {...props} className={`screenshot-input ${props.className}`} ref={ref}>
                {screenshot ? (
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
                ) : undefined}
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
                    {screenshot ? (
                        <>
                            <div
                                className="image-preview"
                                onClick={() => {
                                    setPopup(true);
                                }}
                            >
                                <img src={screenshot.dataURL} />
                            </div>
                        </>
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
                    <input ref={inputRef} type="file" onChange={imageHandler} accept={".jpg, .jpeg, .png"} />
                </div>
                <Popup className="screenshot-popup" nested modal open={popup} onClose={() => setPopup(false)}>
                    <div className="screenshot-input-popup" onClick={() => setPopup(false)}>
                        <img src={screenshot?.dataURL} />
                    </div>
                </Popup>
            </div>
        );
    },
);
