"use client";

import { useEffect, useState } from "react";
import { notifyRefresh } from "../msg-bridge";
import { rajdhani } from "../fonts";

interface ClockProps {
  canvasId: string;
}

export default function TemplateClock({ canvasId }: ClockProps) {
  const [time, setTime] = useState(new Date());

  // Update time every second
  useEffect(() => {
    const interval = setInterval(() => {
      setTime(new Date());
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  // Notify refresh when shit changes
  useEffect(() => {
    notifyRefresh(canvasId);
  }, [time]);

  return (
    <div className="h-full w-full flex flex-col items-center justify-center">
      <p
        className={`text-amber-50 text-8xl font-bold  ${rajdhani.className}`}
      >{`${time.getHours()}:${time.getMinutes()}`}</p>
    </div>
  );
}
