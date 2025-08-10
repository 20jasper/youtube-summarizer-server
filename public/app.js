// @ts-check

import { attachEventSource } from "./sse.js";
import { submitRating } from "./api.js";
import { getParam, setParam, buildSummaryUrl } from "./utils.js";

const PARAM = "url";

/**
 * Initialize the page interactions
 */
function init() {
  const output = /** @type {HTMLElement|null} */ (
    document.querySelector("#output")
  );
  const ytForm = /** @type {HTMLFormElement|null} */ (
    document.querySelector("form#youtube")
  );
  const status = /** @type {HTMLElement|null} */ (
    document.querySelector("#status")
  );
  const ratingsSection = /** @type {HTMLElement|null} */ (
    document.querySelector("#ratings")
  );
  const ratingsForm = /** @type {HTMLFormElement|null} */ (
    document.querySelector("#ratings-form")
  );

  if (!output || !ytForm || !status || !ratingsSection || !ratingsForm)
    throw new Error("missing elements");

  // Submit ratings
  ratingsForm.addEventListener("submit", async (event) => {
    event.preventDefault();
    const formData = new FormData(ratingsForm);
    const rating = /** @type {string|null} */ (formData.get("rating"));
    const message = /** @type {string} */ (formData.get("message") || "");
    const link = getParam(PARAM);

    if (!rating || !link) {
      output.innerHTML = "<p>Please provide a video link first.</p>";
      return;
    }
    const videoId = new URL(link).searchParams.get("v");
    if (!videoId) {
      output.innerHTML = "<p>invalid link</p>";
      return;
    }

    try {
      await submitRating({
        rating,
        message,
        videoId,
      });
    } catch (error) {
      const err = /** @type {Error} */ (error);
      output.innerHTML = `<p>Error submitting feedback: ${err.message}</p>`;
    }
  });

  ytForm.addEventListener("submit", (event) => {
    event.preventDefault();
    const form = /** @type {HTMLFormElement} */ (event.currentTarget);
    const link = /** @type {HTMLInputElement} */ (
      form.elements.namedItem("link")
    );
    const value = link?.value.trim();
    if (!value) return;
    setParam(PARAM, value);
  });

  // If URL param exists, stream immediately
  const link = getParam(PARAM);
  if (link && output) {
    const url = buildSummaryUrl(link);
    status.textContent = "Preparing an amazing summary…";
    output.setAttribute("aria-busy", "true");
    attachEventSource(output, url, {
      onOpen() {
        status.textContent = "Connected";
      },
      onDone() {
        status.textContent = "Done";
        output.setAttribute("aria-busy", "false");
        ratingsSection.classList.remove("hidden");
      },
      onError(err) {
        console.error(err);
        status.textContent = "Connection error";
        output.setAttribute("aria-busy", "false");
      },
    });
  } else if (output) {
    output.innerHTML = "<p>Please provide a video</p>";
  }
}

document.addEventListener("DOMContentLoaded", init);
