import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import {
  getLanguageDirection,
  updateDocumentDirection,
  updateDocumentLanguage,
} from "@/lib/utils/rtl";

// Auto-discover translation files using Vite's glob import
const localeModules = import.meta.glob<{ default: Record<string, unknown> }>(
  "./locales/*/translation.json",
  { eager: true },
);

// Build resources from discovered locale files
const resources: Record<string, { translation: Record<string, unknown> }> = {};
for (const [path, module] of Object.entries(localeModules)) {
  const langCode = path.match(/\.\/locales\/(.+)\/translation\.json/)?.[1];
  if (langCode) {
    resources[langCode] = { translation: module.default };
  }
}

// Initialize i18n with English only.
i18n.use(initReactI18next).init({
  resources,
  lng: "en",
  fallbackLng: "en",
  interpolation: {
    escapeValue: false, // React already escapes values
  },
  react: {
    useSuspense: false, // Disable suspense for SSR compatibility
  },
});

export const syncLanguageFromSettings = async () => {
  if (i18n.language !== "en") {
    await i18n.changeLanguage("en");
  }
};

// Run language sync on init
syncLanguageFromSettings();

// Listen for language changes to update HTML dir and lang attributes
i18n.on("languageChanged", (lng) => {
  const dir = getLanguageDirection(lng);
  updateDocumentDirection(dir);
  updateDocumentLanguage(lng);
});

// Re-export RTL utilities for convenience
export { getLanguageDirection, isRTLLanguage } from "@/lib/utils/rtl";

export default i18n;
