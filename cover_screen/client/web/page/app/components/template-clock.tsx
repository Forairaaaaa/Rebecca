"use client";

import { useEffect, useState } from "react";
import { notifyRefresh } from "../msg-bridge";
import { rajdhani } from "../fonts";

interface ClockProps {
  canvasId: string;
}

export default function TemplateClock({ canvasId }: ClockProps) {
  const getTime = (time: Date) => {
    return `${time.getHours()}:${time.getMinutes()}`;
  };

  const [time, setTime] = useState("");

  // Check if time(hh:mm) changes every second
  useEffect(() => {
    const interval = setInterval(() => {
      const now = getTime(new Date());
      if (now !== time) {
        setTime(now);
      }
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  // Notify refresh when time changes
  useEffect(() => {
    notifyRefresh(canvasId);
  }, [time]);

  return (
    <div className="h-full w-full flex flex-col items-center justify-center">
      <p className={`text-amber-50 text-8xl font-bold  ${rajdhani.className}`}>
        {time}
      </p>
    </div>
  );
}
