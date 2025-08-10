// @ts-check

/**
 * Submit a rating for a summary
 * @param {{ rating: string, message?: string, videoId: string }} payload
 * @returns {Promise<boolean>} response.ok
 */
export async function submitRating(payload) {
  const res = await fetch(`/summary/rating`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  return res.ok;
}
