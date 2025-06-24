"use client";

export function notifyRefresh(canvasId: string) {
  fetch(`/api/refresh/${canvasId}`)
    .then((res) => {
      console.log("refresh canvas", canvasId, res);
    })
    .catch((err) => {
      console.error("refresh canvas", canvasId, err);
    });
}
