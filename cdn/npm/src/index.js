const API_HOST = "https://cdn.jsdelivr.net/npm"

async function handleRequest(event) {
	const url = new URL(event.request.url)
	const pathname = url.pathname
	const search = url.search
	const pathWithParams = pathname + search
	let params = pathWithParams.replace("npm@", "@");

	if (!params.endsWith("+esm")) {
		params = params+"/+esm";
	} else {
		params = "/"+params.split("/").slice(2).join("/");
	}

	if (params.startsWith("/npm/")) {
		params = params.replace("/npm/","/");
	}

	console.log("params", params);
	return forwardRequest(event, params)
}

async function forwardRequest(event, pathWithSearch) {
	const request = new Request(event.request);
	console.log("URL: ",`${API_HOST}${pathWithSearch}`);
	let response = await fetch(`${API_HOST}${pathWithSearch}`, request);
	if(response.status == 302){
		response = await fetch(response.url);
	}
	return response
}

addEventListener("fetch", (event) => {
	event.passThroughOnException()
	event.respondWith(handleRequest(event))
})