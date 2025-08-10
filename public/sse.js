// @ts-check

// @ts-ignore
import * as smd from "https://cdn.jsdelivr.net/npm/streaming-markdown/smd.min.js";

/**
 * Streams markdown from `url` into `el`.
 *
 * @param {HTMLElement} el
 * @param {string} url
 * @param {{ onOpen?: () => void, onDone?: () => void, onError?: (err: any) => void }} [opts]
 * @returns {EventSource}
 */
export function attachEventSource(el, url, opts = {}) {
  const renderer = smd.default_renderer(el);
  const parser = smd.parser(renderer);

  const evtSource = new EventSource(url);

  evtSource.addEventListener("message", (event) => {
    const data = JSON.parse(event.data);
    const { kind, message } = data;
    switch (kind) {
      case "done": {
        evtSource.close();
        opts.onDone?.();
        break;
      }
      case "message": {
        smd.parser_write(parser, message ?? "");
        break;
      }
      case "error": {
        console.error("Stream error:", data);
        opts.onError?.(data);
        break;
      }
      default: {
        console.warn("Unknown event kind:", kind);
      }
    }
  });

  evtSource.onerror = (err) => {
    console.error("EventSource failed:", err);
    opts.onError?.(err);
  };
  evtSource.onopen = () => {
    opts.onOpen?.();
  };

  return evtSource;
}
