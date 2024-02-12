import remarkGfm from "remark-gfm";
import Markdown, { Options } from "react-markdown";
import "../styling/markdown.css";
import rehypeHighlight from "rehype-highlight";

/**
 * Tiny wrapper around {@link Markdown `<Markdown />`} which adds the {@link remarkGfm `remarkGfm`} plugin.
 *
 * I.e. this components body will be rendered as GitHub flavored markdown.
 */
export function GithubMarkdown(props: Readonly<Options>) {
    return Markdown({
        ...props,
        remarkPlugins: [remarkGfm, ...(props.remarkPlugins || [])],
        rehypePlugins: [rehypeHighlight, ...(props.rehypePlugins || [])],
        className: "markdown",
    });
}
