export interface ThemeColors {
  /** Цвет активных элементов по вертикали (default: oklch(0.65 0.25 290) - Фиолетовый) */
  primary?: string;
  /** Цвет активных элементов по горизонтали (default: oklch(0.85 0.15 190) - Циан) */
  secondary?: string;
  /** Цвет фона игры (default: oklch(0.1 0.01 260) - Почти черный) */
  background?: string;
  /** Основной цвет текста (default: oklch(0.98 0 0) - Почти белый) */
  foreground?: string;
}

export interface CyberHackConfig {
  /** Ссылка для редиректа после игры (например: '/success') */
  redirect_url: string;
  /** Базовая награда за демона (умножается на уровень) */
  base_value: number;
  /** Время на взлом в секундах (Таймер стартует по первому клику) */
  time_limit: number;
  /** Язык интерфейса ('ru' | 'en'). По умолчанию: 'ru' */
  locale?: "ru" | "en";
  /** Опциональная кастомизация цветов (можно использовать hex, rgb, oklch) */
  theme?: ThemeColors;
}

/** Инициализирует и рендерит игру в указанный HTML элемент */
export function initCyberHack(elementId: string, configJson: string): void;

// Удобная JS-обертка, чтобы не писать JSON.stringify вручную
export function mountGame(elementId: string, config: CyberHackConfig): void;
