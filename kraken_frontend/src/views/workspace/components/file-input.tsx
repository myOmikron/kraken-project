import React, { DragEvent, forwardRef } from "react";
import { toast } from "react-toastify";
import Popup from "reactjs-popup";
import { Api } from "../../../api/api";
import "../../../styling/file-input.css";
import CloseIcon from "../../../svg/close";
import { handleApiError } from "../../../utils/helper";
import { WORKSPACE_CONTEXT } from "../workspace";

export type FileInputProps = {
    /** The image as either a {@link File} or an uuid for an uploaded image*/
    file: File | string | undefined;

    /** Callback invoked when the user changed the selected image */
    onChange: (newFile: File | undefined) => void;

    /** Preview images & restrict file types to images */
    image?: boolean;

    /** Toggles the component between compact and bigger */
    shortText?: boolean;
} & Omit<React.HTMLProps<HTMLDivElement>, "onChange">;

export const FileInput = forwardRef<HTMLDivElement, FileInputProps>(
    ({ file, image: isImage, onChange, shortText, ...props }, ref) => {
        const {
            workspace: { uuid: workspace },
        } = React.useContext(WORKSPACE_CONTEXT);

        const [drag, setDrag] = React.useState<boolean>(false);
        const [popup, setPopup] = React.useState<boolean>(false);

        const [blobURL, setBlobURL] = React.useState("");
        React.useEffect(() => {
            if (file !== undefined && typeof file !== "string") {
                const url = URL.createObjectURL(file);
                setBlobURL(url);
                return () => URL.revokeObjectURL(url);
            }
        }, [file]);
        const fullFileUrl = typeof file === "object" ? blobURL : file && `/api/v1/workspace/${workspace}/files/${file}`;
        const thumbnailUrl =
            isImage &&
            (typeof file === "object" ? blobURL : file && `/api/v1/workspace/${workspace}/files/${file}/thumbnail`);

        const inputRef = React.useRef<HTMLInputElement>(null);

        const fileHandler = (file: File | undefined) => {
            setDrag(false);

            if (!file) {
                // TODO: when exactly is this branch even called?
                return void toast.error("No file selected");
            }

            if (isImage && !file.type.includes("image")) {
                return void toast.error("File must be an image!");
            }

            if (file.size > 100 * 1024 * 1024) {
                return void toast.error("File too large");
            }

            onChange(file);
            return true;
        };

        const dropHandler = (e: DragEvent<HTMLInputElement>) => {
            e.preventDefault();
            if (inputRef.current && fileHandler(e.dataTransfer.files?.[0])) {
                inputRef.current.files = e.dataTransfer.files;
            }
        };

        return (
            <div
                {...props}
                className={`file-input ${props.className}`}
                ref={ref}
                onDropCapture={dropHandler}
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
                {file && (
                    <button
                        title={isImage ? "Remove image" : "Remove file"}
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
                <div className="content">
                    {thumbnailUrl ? (
                        <div
                            className="image-preview"
                            onClick={() => {
                                setPopup(true);
                            }}
                        >
                            <img src={thumbnailUrl} alt={"Screenshot Thumbnail"} />
                        </div>
                    ) : fullFileUrl ? (
                        <a className="file-preview" target="_blank" href={fullFileUrl}>
                            {typeof file === "object" ? file.name : "View File"}
                        </a>
                    ) : (
                        <>
                            {props.children}
                            {drag ? (
                                <span>
                                    {isImage
                                        ? shortText
                                            ? "Drop image"
                                            : "Drop image here"
                                        : shortText
                                          ? "Drop attachment"
                                          : "Drop attachment here"}
                                </span>
                            ) : (
                                <span>
                                    {isImage
                                        ? shortText
                                            ? "Upload image"
                                            : "Drag your image here or click in this area"
                                        : shortText
                                          ? "Upload attachment"
                                          : "Drag your file here or click in this area"}
                                </span>
                            )}
                        </>
                    )}

                    <input
                        ref={inputRef}
                        type="file"
                        onChange={(event) => {
                            const file = event.target.files?.[0];
                            fileHandler(file);
                        }}
                        accept={isImage ? ".jpg, .jpeg, .png" : "*"}
                    />
                </div>
                <Popup className="screenshot-popup" nested modal open={popup} onClose={() => setPopup(false)}>
                    <div className="file-input-popup" onClick={() => setPopup(false)}>
                        <img src={fullFileUrl} alt={"Screenshot"} />
                    </div>
                </Popup>
            </div>
        );
    },
);

export type UploadingFileInputProps = {
    onUploaded?: (uuid: string | undefined) => void;
    onChange?: FileInputProps["onChange"] | undefined;
} & Omit<FileInputProps, "onChange">;

/**
 * FileInput component that automatically uploads the image/file when selected
 * to the server and shows an upload animation while doing so.
 */
export const UploadingFileInput = forwardRef<HTMLDivElement, UploadingFileInputProps>(
    ({ onUploaded, onChange, ...props }, ref) => {
        const {
            workspace: { uuid: workspace },
        } = React.useContext(WORKSPACE_CONTEXT);

        const [uploading, setUploading] = React.useState<File>();

        return (
            <FileInput
                {...props}
                ref={ref}
                className={`${props.className} ${uploading ? "uploading" : ""}`}
                file={uploading ?? props.file}
                onChange={(newFile) => {
                    onChange?.(newFile);
                    if (newFile === undefined) {
                        onUploaded?.(undefined);
                    } else {
                        setUploading(newFile);
                        (props.image ? Api.workspaces.files.uploadImage : Api.workspaces.files.uploadFile)(
                            workspace,
                            newFile.name,
                            newFile,
                        ).then((r) => {
                            setUploading(undefined);
                            handleApiError(r, (r) => {
                                onUploaded?.(r.uuid);
                            });
                        });
                    }
                }}
            />
        );
    },
);
