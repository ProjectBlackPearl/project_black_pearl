import { readTextFile, BaseDirectory } from '@tauri-apps/api/fs';
import { invoke } from '@tauri-apps/api/tauri';
import { locale } from '../locale/i18n';

// TS Function
// - Gets the current locale from config.json
export async function getCurrentLocale() {
    const config: string = await readTextFile('config.json', {
        dir: BaseDirectory.AppLocalData,
    }).catch(() => {
        log(0, 'Failed to read config.json');

        return '';
    });

    let configParsed = JSON.parse(config);
    return configParsed.currentLang as string;
}

// TS Function
// - Loads the current locale
export async function loadLocale() {
    locale.set(await getCurrentLocale());
}

// TS Function -> Rust Function
// - Logs a message to the Rust backend
export function log(logLevel: number, logMessage: string) {
    const levels: string[] = ['ERROR', 'WARNING', 'INFO'];
    console.log(`[${levels[logLevel]}]: ${logMessage}`);
    invoke('log', {
        logLevel: logLevel,
        logMessage: `From TS: ${logMessage}`,
    });
}
