import { fetch, ResponseType } from '@tauri-apps/api/http';

// TS Function
// - Gets news from a json file on github and returns the parsed version of it
export async function pbpNews() {
	// TODO: Change the URI to a file named CHANGELOG.json on the repo
	const uri =
		'https://raw.githubusercontent.com/Brisolo32/test/main/CHANGELOG.json';
	const response: any = await fetch(uri, {
		responseType: ResponseType.Text,
		method: 'GET',
	});

	const responseParsed = JSON.parse(response.data);
	return responseParsed;
}
