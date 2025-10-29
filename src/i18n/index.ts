import { createI18n } from "vue-i18n";
import enUS, { type LocaleStrings } from "./locales/en-US";
import zhCN from "./locales/zh-CN";

export type { LocaleStrings };

const messages: Record<string, LocaleStrings> = {
  "en-US": enUS,
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
  { key: "en-US", label: "English" },
  { key: "zh-CN", label: "简体中文" },
];
