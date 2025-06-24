"use client";

import { useEffect } from "react";
import { startMsgBridge } from "./msg-bridge";
import TemplateClock from "./components/template-clock";

export default function Home() {
  useEffect(() => {
    console.log("start msg bridge");
    startMsgBridge();
  }, []);

  return (
    <main className="flex-col">
      <div id="cover-screen-0" className="w-[280px] h-[240px]">
        <TemplateClock canvasId="cover-screen-0" />
      </div>
      <div id="cover-screen-1" className="w-[280px] h-[240px]"></div>
    </main>
  );
}
