"use client";
import React, { useEffect, useRef } from "react";
//@ts-ignore
import init, { initCyberHack } from "./cyberhack.js";
//@ts-ignore
import "./cyberhack.css";

export interface CyberhackTheme {
  primary?: string;
  secondary?: string;
  background?: string;
  foreground?: string;
}

export interface CyberhackResult {
  completed_targets: number[];
  buffer: string[];
  total_coins: number;
}

export interface CyberhackProps {
  redirectUrl: string;
  baseValue: number;
  timeLimit: number;
  locale?: "ru" | "en";
  theme?: CyberhackTheme;
  onComplete?: (result: CyberhackResult) => void;
}

export const Cyberhack: React.FC<CyberhackProps> = ({
  redirectUrl,
  baseValue,
  timeLimit,
  locale = "ru",
  theme,
  onComplete,
}) => {
  const containerId = useRef(
    `cyberhack-${Math.random().toString(36).substring(7)}`,
  );
  const initialized = useRef(false);

  useEffect(() => {
    if (!initialized.current) {
      init().then(() => {
        const config = {
          redirect_url: redirectUrl,
          base_value: baseValue,
          time_limit: timeLimit,
          locale,
          theme,
        };
        initCyberHack(containerId.current, JSON.stringify(config));
        initialized.current = true;
      });
    }

    const handleMessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as CyberhackResult;
        if (data.total_coins !== undefined && onComplete) {
          onComplete(data);
        }
      } catch (e) {}
    };

    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [redirectUrl, baseValue, timeLimit, locale, theme, onComplete]);

  return (
    <div className="dark w-full h-full flex items-center justify-center">
      <div id={containerId.current}></div>
    </div>
  );
};
