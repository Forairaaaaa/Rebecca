"use client"

import { useEffect, useState } from "react";
import { notifyRefresh } from "../msg-bridge";

interface ClockProps {
    canvasId: string;
}

export default function TemplateClock({ canvasId }: ClockProps) {
    const [time, setTime] = useState(new Date());

    useEffect(() => {
        const interval = setInterval(() => {
            setTime(new Date());
        }, 1000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        notifyRefresh(canvasId);
    }, [time]);

    return (
        <div className="w-[280px] h-[240px] bg-white">
            <p className="text-black">{time.toLocaleTimeString()}</p>
            <div className="w-[100px] h-[100px] shadow-lg"></div>
        </div>
    )
}
