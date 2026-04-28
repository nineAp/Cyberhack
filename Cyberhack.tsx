"use client";
import React, { useEffect, useRef, useState } from "react";

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
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const initialized = useRef(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Эффект Canvas для кибер-фона (Hex-дождь с поддержкой динамической темы)
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animationFrameId: number;
    const fontSize = 16;
    let columns = 0;
    let drops: number[] = [];
    const chars = "0123456789ABCDEF".split("");

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
      columns = Math.floor(canvas.width / fontSize);
      drops = [];
      for (let x = 0; x < columns; x++) {
        drops[x] = Math.random() * canvas.height;
      }
    };

    window.addEventListener("resize", resize);
    resize();

    const draw = () => {
      // Полупрозрачный фон для эффекта "шлейфа" с учетом темы
      ctx.globalAlpha = 0.15;
      ctx.fillStyle = theme?.background || "#05020a";
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      ctx.font = `${fontSize}px monospace`;

      for (let i = 0; i < drops.length; i++) {
        const text = chars[Math.floor(Math.random() * chars.length)];
        const x = i * fontSize;
        const y = drops[i] * fontSize;

        // Динамические цвета символов (Primary и Secondary из темы)
        if (Math.random() > 0.98) {
          ctx.globalAlpha = 0.9;
          ctx.fillStyle = theme?.primary || "#8b3dff";
        } else {
          ctx.globalAlpha = 0.4;
          ctx.fillStyle = theme?.secondary || "#00f0ff";
        }

        ctx.fillText(text, x, y);

        if (y > canvas.height && Math.random() > 0.975) {
          drops[i] = 0;
        }
        drops[i]++;
      }
      animationFrameId = requestAnimationFrame(draw);
    };

    draw();

    return () => {
      window.removeEventListener("resize", resize);
      cancelAnimationFrame(animationFrameId);
    };
  }, [theme]);

  // Инициализация WASM Yew (с обходом строгой статической проверки сборщика)
  useEffect(() => {
    if (!initialized.current) {
      // Изолируем пути к файлам через переменные, чтобы сборщик не падал,
      // если WASM файл еще физически не существует.
      const wasmModulePath = "./cyberhack.js";
      const cssModulePath = "./cyberhack.css";

      Promise.all([
        //@ts-ignore
        import("cyberhack/wasm"), // Мы прописали это в exports package.json
        //@ts-ignore
        import("cyberhack/style").catch(() => {}),
      ])
        .then(([wasmModule]) => {
          const init = wasmModule.default;
          const initCyberHack = wasmModule.initCyberHack;

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
        })
        .catch((e) => {
          console.warn("WASM-модуль еще не собран. Ожидание сборки...", e);
          setLoadError(
            "Библиотека WASM не найдена. Убедитесь, что выполнен скрипт сборки.",
          );
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
    <div
      className="dark relative w-full h-screen overflow-hidden bg-[#05020a] font-mono"
      style={{
        backgroundColor: theme?.background || "#05020a",
        color: theme?.foreground || "#ffffff",
      }}
    >
      {loadError && (
        <div className="absolute top-0 left-0 w-full bg-red-900 text-white text-xs p-2 text-center z-[100] border-b border-red-500">
          {loadError}
        </div>
      )}

      {/* Анимированный Canvas фон */}
      <canvas ref={canvasRef} className="absolute inset-0 z-0 opacity-80" />

      {/* Радиальный градиент для фокусировки в центре */}
      <div
        className="absolute inset-0 z-0 pointer-events-none"
        style={{
          background: `radial-gradient(circle at center, transparent 0%, ${theme?.background || "rgba(5,2,10,0.8)"} 100%)`,
        }}
      ></div>

      {/* Yew App Контейнер */}
      <div
        id={containerId.current}
        className="relative z-10 w-full h-full flex items-center justify-center overflow-auto"
      ></div>

      {/* Оверлей монитора CRT */}
      <div className="crt-overlay pointer-events-none absolute inset-0 z-50"></div>
    </div>
  );
};
