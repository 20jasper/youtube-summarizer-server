// @ts-check

export const PARAM = "url";
export const STREAM_URL_BASE = "/summary";

/**
 * Get a URL param value
 * @param {string} key
 */
export function getParam(key) {
  return new URLSearchParams(window.location.search).get(key);
}

/**
 * Set a URL param value by reloading the page with the new query string
 * @param {string} key
 * @param {string} value
 */
export function setParam(key, value) {
  const encoded = encodeURIComponent(value);
  window.location.search = `?${key}=${encoded}`;
}

/**
 * Build the streaming summary URL for a given link
 * @param {string} link
 */
export function buildSummaryUrl(link) {
  return `${STREAM_URL_BASE}?${PARAM}=${encodeURIComponent(link ?? "")}`;
}
