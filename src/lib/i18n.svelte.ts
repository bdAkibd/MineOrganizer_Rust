import localesMeta from '../locales/locales.json';

import en from '../locales/en.json';
import ru from '../locales/ru.json';
import de from '../locales/de.json';
import es from '../locales/es.json';
import fr from '../locales/fr.json';
import uk from '../locales/uk.json';
import it from '../locales/it.json';
import zh_CN from '../locales/zh_CN.json';
import zh_TW from '../locales/zh_TW.json';
import ja from '../locales/ja.json';
import pt from '../locales/pt.json';

export const SUPPORTED_LOCALES = Object.keys(localesMeta) as Array<keyof typeof localesMeta>;
export const LOCALE_NAMES = localesMeta as Record<string, string>;

const TRANSLATIONS: Record<string, Record<string, string>> = {
  en,
  ru,
  de,
  es,
  fr,
  uk,
  it,
  zh_CN,
  zh_TW,
  ja,
  pt,
};

class I18nState {
  currentLocale = $state<string>('en');

  constructor() {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('mineorganizer_locale');
      if (saved && saved in TRANSLATIONS) {
        this.currentLocale = saved;
      } else {
        const navLang = navigator.language.replace('-', '_');
        if (navLang in TRANSLATIONS) {
          this.currentLocale = navLang;
        } else {
          const short = navLang.split('_')[0];
          if (short in TRANSLATIONS) {
            this.currentLocale = short;
          }
        }
      }
    }
  }

  setLocale(locale: string) {
    if (locale in TRANSLATIONS) {
      this.currentLocale = locale;
      if (typeof window !== 'undefined') {
        localStorage.setItem('mineorganizer_locale', locale);
      }
    }
  }

  t(key: string, params?: Record<string, string | number>): string {
    const localeDict = TRANSLATIONS[this.currentLocale] || TRANSLATIONS['en'];
    let template = localeDict[key] ?? TRANSLATIONS['en']?.[key] ?? key;
    if (params) {
      for (const [k, val] of Object.entries(params)) {
        template = template.replaceAll(`{${k}}`, String(val));
      }
    }
    return template;
  }
}

export const i18n = new I18nState();
export const t = (key: string, params?: Record<string, string | number>) => i18n.t(key, params);
