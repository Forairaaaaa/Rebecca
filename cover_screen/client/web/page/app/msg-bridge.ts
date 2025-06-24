"use client";

let _ws: WebSocket | null = null;

export function startMsgBridge() {
  _ws = new WebSocket("ws://" + window.location.host + "/ws");
}

export function sendMsg(msg: string) {
  if (_ws) {
    try {
      _ws.send(msg);
    } catch (error) {
      console.error("send msg error", error);
    }
  }
}

export function notifyRefresh(canvasId: string) {
  console.log("notify refresh", canvasId);
  sendMsg(JSON.stringify({ action: "refresh", canvasId }));
}
