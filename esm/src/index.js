const API_HOST = "https://esm.sh"

async function handleRequest(event) {
	const url = new URL(event.request.url)
	const pathname = url.pathname
	const search = url.search
	const pathWithParams = pathname + search
	let params = pathWithParams.replace("/tools", "")
	if (params.split("/").length > 2) {
		params = "/"+params.split("/").slice(2).join("/");
	}
	return forwardRequest(event, params)
}

async function forwardRequest(event, pathWithSearch) {
	const request = new Request(event.request)
	console.log(`${API_HOST}${pathWithSearch}`);
	return await fetch(`${API_HOST}${pathWithSearch}`, request)
}

addEventListener("fetch", (event) => {
	event.passThroughOnException()
	event.respondWith(handleRequest(event))
})