// @ts-check

// @ts-ignore
import * as smd from "https://cdn.jsdelivr.net/npm/streaming-markdown/smd.min.js";

/**
 * streams markdown from `url` into `el`
 *
 * @param {HTMLElement} el
 * @param {string} url
 */
export function attachEventSource(el, url) {
  const renderer = smd.default_renderer(el);
  const parser = smd.parser(renderer);

  const evtSource = new EventSource(url);

  evtSource.addEventListener("message", (event) => {
    console.log(event);
    const data = JSON.parse(event.data);
    const { kind, message } = data;
    switch (kind) {
      case "done":
        console.log("Stream done");
        evtSource.close();
        break;
      case "message":
        console.log("Stream message:", data);
        smd.parser_write(parser, message ?? "");
        break;
      case "error":
        console.error("Stream error:", data);
        break;
      default:
        throw new Error("Unknown event kind: " + kind);
    }
  });

  evtSource.onerror = (err) => {
    console.error("EventSource failed:", err);
  };
  evtSource.onopen = () => {
    console.log("Connection opened");
  };
}
