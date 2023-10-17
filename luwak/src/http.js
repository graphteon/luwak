class Context {
    #conn
    #httpConn
    #requestEvent
    #params
    #url
    body = undefined
    #extra = {}
    error = undefined
    #postProcessors = new Set()
    constructor(conn, httpConn, requestEvent, params, url, req, hasRoute) {
        this.#conn = conn
        this.#httpConn = httpConn
        this.#requestEvent = requestEvent
        this.#params = params
        this.#url = url
        this.req = req
        this.res.status = hasRoute ? 200 : 404
    }
    res = {
        body: undefined,
        headers: new Headers(),
        status: 200,
        statusText: ""
    }
    get conn() {
        return this.#conn
    }
    get httpConn() {
        return this.#httpConn
    }
    get requestEvent() {
        return this.#requestEvent
    }
    get params() {
        return this.#params
    }
    get url() {
        return this.#url
    }
    get extra() {
        return this.#extra
    }
    get postProcessors() {
        return this.#postProcessors
    }
    redirect(url) {
        this.postProcessors.add(ctx => {
            ctx.res.headers.set("Location", url)
            ctx.res.status = 301
        })
    }
}

// Adapted from https://github.com/lukeed/regexparam/blob/master/src/index.js
function parse(str, loose) {
    if (str instanceof RegExp) return { keys: [], pattern: str }
    var c,
        o,
        tmp,
        ext,
        keys = [],
        pattern = "",
        arr = str.split("/")
    arr[0] || arr.shift()

    while ((tmp = arr.shift())) {
        c = tmp[0]
        if (c === "*") {
            keys.push("wild")
            pattern += "/(.*)"
        } else if (c === ":") {
            o = tmp.indexOf("?", 1)
            ext = tmp.indexOf(".", 1)
            keys.push(tmp.substring(1, !!~o ? o : !!~ext ? ext : tmp.length))
            pattern += !!~o && !~ext ? "(?:/([^/]+?))?" : "/([^/]+?)"
            if (!!~ext) pattern += (!!~o ? "?" : "") + "\\" + tmp.substring(ext)
        } else {
            pattern += "/" + tmp
        }
    }

    return {
        keys: keys,
        pattern: new RegExp("^" + pattern + (loose ? "(?=$|/)" : "/?$"), "i")
    }
}

class Server {
    // NOTE: This is transpiled into the constructor, therefore equivalent to this.routes = [];
    #routes = []

    // NOTE: Using .bind can significantly increase perf compared to arrow functions.
    all = this.#add.bind(this, "ALL")
    get = this.#add.bind(this, "GET")
    head = this.#add.bind(this, "HEAD")
    patch = this.#add.bind(this, "PATCH")
    options = this.#add.bind(this, "OPTIONS")
    connect = this.#add.bind(this, "CONNECT")
    delete = this.#add.bind(this, "DELETE")
    trace = this.#add.bind(this, "TRACE")
    post = this.#add.bind(this, "POST")
    put = this.#add.bind(this, "PUT")

    use(...handlers) {
        this.#routes.push({
            keys: [],
            method: "ALL",
            handlers
        })
        return this
    }

    #add(method, route, ...handlers) {
        var { keys, pattern } = parse(route)
        this.#routes.push({
            keys,
            method,
            handlers,
            pattern
        })
        return this
    }

    async #middlewareHandler(fns, fnIndex, ctx) {
        if (fns[fnIndex] !== undefined) {
            try {
                await fns[fnIndex](
                    ctx,
                    async () => await this.#middlewareHandler(fns, fnIndex + 1, ctx)
                )
            } catch (e) {
                ctx.error = e
            }
        }
    }

    async #handleRequest(conn) {
        try {
            const httpConn = Deno.serveHttp(conn)
            for await (const requestEvent of httpConn) {
                const req = requestEvent.request
                const url = new URL(requestEvent.request.url)
                const requestHandlers = []
                const params = {}
                const len = this.#routes.length
                var hasRoute = false
                for (var i = 0; i < len; i++) {
                    const r = this.#routes[i]
                    const keyLength = r.keys.length
                    var matches = null
                    if (
                        r.pattern === undefined ||
                        (req.method === r.method &&
                            (matches = r.pattern.exec(url.pathname)))
                    ) {
                        if (r.pattern) {
                            hasRoute = true
                            if (keyLength > 0) {
                                if (matches) {
                                    var inc = 0
                                    while (inc < keyLength) {
                                        params[r.keys[inc]] = decodeURIComponent(matches[++inc])
                                    }
                                }
                            }
                        }
                        requestHandlers.push(...r.handlers)
                    }
                }
                var ctx = new Context(
                    conn,
                    httpConn,
                    requestEvent,
                    params,
                    url,
                    req,
                    hasRoute
                )
                await this.#middlewareHandler(requestHandlers, 0, ctx)
                if (!ctx.error) {
                    try {
                        for (const p of ctx.postProcessors) {
                            await p(ctx)
                        }
                    } catch (e) {
                        ctx.error = e
                    }
                }
                if (ctx.error) {
                    ctx.res.status = 500
                    ctx.res.headers.set("Content-Type", "application/json")
                    ctx.res.body = JSON.stringify({
                        msg: ctx.error.message || ctx.error,
                        stack: ctx.error.stack
                    })
                }
                await requestEvent.respondWith(
                    new Response(ctx.res.body, {
                        headers: ctx.res.headers,
                        status: ctx.res.status,
                        statusText: ctx.res.statusText
                    })
                )
            }
        } catch (e) {
            console.log(e)
        }
    }

    async listen(serverParams) {
        const server =
            serverParams.certFile || serverParams.port === 443
                ? Deno.listenTls(serverParams)
                : Deno.listen(serverParams)
        try {
            for await (const conn of server) {
                this.#handleRequest(conn)
            }
        } catch (e) {
            console.log(e)
            if (e.name === "NotConnected") {
                await this.listen(serverParams)
            }
        }
    }
}

const reqParsers = {}
const resParsers = {
    json: ctx => {
        ctx.res.headers.set("Content-Type", "application/json")
        ctx.res.body = JSON.stringify(ctx.res.body)
    },
    html: ctx => {
        ctx.res.headers.set("Content-Type", "text/html")
        ctx.res.body = ctx.res.body
    },
    javascript: ctx => {
        ctx.res.headers.set("Content-Type", "application/javascript")
        ctx.res.body = ctx.res.body
    }
}

function res(type) {
    return async (ctx, next) => {
        if (!resParsers[type]) {
            throw new Error(`Response body parser: '${type}' not supported.`)
        }
        ctx.postProcessors.add(resParsers[type])
        await next()
    }
}

function req(type) {
    return async (ctx, next) => {
        if (ctx.req.bodyUsed) {
            return
        }
        //@ts-ignore
        if (ctx.req[type]) {
            //native JavaScript parser
            //@ts-ignore
            ctx.body = await ctx.req[type]()
        } else if (reqParsers[type]) {
            await reqParsers[type](ctx)
        } else {
            throw new Error(`Request body parser: '${type}' not supported.`)
        }
        await next()
    }
}

globalThis.Luwak.http = {
    Server,
    req,
    res,
    serve: (ops, handlers) => {
        Deno.serve(ops, handlers)
    }
}

