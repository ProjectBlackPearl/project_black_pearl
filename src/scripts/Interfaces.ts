export interface Config {
	currentLang: string;
}

export interface TempScrapers {
	scrapers: Scraper[];
}

export interface Scraper {
	name: string;
	location: string;
}

export interface SearchResults {
	response: Response[];
}

export interface Response {
	title: string;
	urls: string[];
}

export interface Game {
	id: number;
	name: string;
	exe_path: string;
	description: string;
	image: string;
}
