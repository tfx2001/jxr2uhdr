import { createI18n } from "vue-i18n";
import en from "./en";
import zhCN from "./zh-CN";

export type MessageSchema = typeof en;

const i18n = createI18n<[MessageSchema], "en" | "zh-CN">({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: "en",
  messages: {
    en,
    "zh-CN": zhCN,
  },
});

function detectLocale(): "en" | "zh-CN" {
  const stored = localStorage.getItem("locale");
  if (stored === "en" || stored === "zh-CN") return stored;
  const lang = navigator.language;
  if (lang.startsWith("zh")) return "zh-CN";
  return "en";
}

export function setLocale(locale: "en" | "zh-CN") {
  (i18n.global.locale as unknown as { value: "en" | "zh-CN" }).value = locale;
  document.documentElement.lang = locale === "zh-CN" ? "zh-CN" : "en";
  localStorage.setItem("locale", locale);
}

export default i18n;
