import { createI18n } from "vue-i18n";
import deDE from "./locales/de-DE";
import enUS, { type LocaleStrings } from "./locales/en-US";
import esES from "./locales/es-ES";
import frFR from "./locales/fr-FR";
import itIT from "./locales/it-IT";
import jaJP from "./locales/ja-JP";
import koKR from "./locales/ko-KR";
import ptBR from "./locales/pt-BR";
import ruRU from "./locales/ru-RU";
import zhCN from "./locales/zh-CN";

export type { LocaleStrings };

const messages: Record<string, LocaleStrings> = {
  "de-DE": deDE,
  "en-US": enUS,
  "es-ES": esES,
  "fr-FR": frFR,
  "it-IT": itIT,
  "ja-JP": jaJP,
  "ko-KR": koKR,
  "pt-BR": ptBR,
  "ru-RU": ruRU,
  "zh-CN": zhCN,
};

type LocaleKey = keyof typeof messages;

const navigatorLocale =
  typeof navigator !== "undefined" ? navigator.language : undefined;

export const LANGUAGE_FALLBACK = "en-US";

export const i18n = createI18n({
  fallbackLocale: LANGUAGE_FALLBACK,
  legacy: false,
  locale: navigatorLocale ?? LANGUAGE_FALLBACK,
  messages,
});

export function setI18nLocale(locale: string) {
  i18n.global.locale.value = locale as LocaleKey;
}

export function getI18nLocale(): LocaleKey {
  return i18n.global.locale.value;
}

export const supportedLocales: Array<{ key: string; label: string }> = [
  { key: "de-DE", label: "Deutsch" },
  { key: "en-US", label: "English" },
  { key: "es-ES", label: "Español" },
  { key: "fr-FR", label: "Français" },
  { key: "it-IT", label: "Italiano" },
  { key: "ja-JP", label: "日本語" },
  { key: "ko-KR", label: "한국어" },
  { key: "pt-BR", label: "Português (Brasil)" },
  { key: "ru-RU", label: "Русский" },
  { key: "zh-CN", label: "简体中文" },
];
