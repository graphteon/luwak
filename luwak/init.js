globalThis.Luwak = () => {
    return Deno;
}

globalThis.Luwak.version = "0.8.0";

globalThis.require = async (package_name, exports = null) => {
    const pkg = exports ? `npm:${package_name}?exports=${exports.join(",")}` : `npm:${package_name}`;
    const module = await import(pkg);
    if (exports) {
        return module;
    }
    return module.default;
}

globalThis.npmExports = (package_name, exports) => {
    const pkg = exports ? `npm:${package_name}?exports=${exports.join(",")}` : `npm:${package_name}`;
    return pkg;
}
class AssertionError extends Error {
    constructor(message) {
        super(message);
        this.name = "AssertionError";
    }
}
function assert(expr, msg = "") {
    if (!expr) {
        throw new AssertionError(msg);
    }
}
const { hasOwn } = Object;
function get(obj, key) {
    if (hasOwn(obj, key)) {
        return obj[key];
    }
}
function getForce(obj, key) {
    const v = get(obj, key);
    assert(v !== undefined);
    return v;
}
function isNumber(x) {
    if (typeof x === "number")
        return true;
    if (/^0x[0-9a-f]+$/i.test(String(x)))
        return true;
    return /^[-+]?(?:\d+(?:\.\d*)?|\.\d+)(e[-+]?\d+)?$/.test(String(x));
}
function hasKey(obj, keys) {
    let o = obj;
    keys.slice(0, -1).forEach((key) => {
        var _a;
        o = ((_a = get(o, key)) !== null && _a !== void 0 ? _a : {});
    });
    const key = keys[keys.length - 1];
    return hasOwn(o, key);
}
/** Take a set of command line arguments, optionally with a set of options, and
 * return an object representing the flags found in the passed arguments.
 *
 * By default, any arguments starting with `-` or `--` are considered boolean
 * flags. If the argument name is followed by an equal sign (`=`) it is
 * considered a key-value pair. Any arguments which could not be parsed are
 * available in the `_` property of the returned object.
 *
 * By default, the flags module tries to determine the type of all arguments
 * automatically and the return type of the `parse` method will have an index
 * signature with `any` as value (`{ [x: string]: any }`).
 *
 * If the `string`, `boolean` or `collect` option is set, the return value of
 * the `parse` method will be fully typed and the index signature of the return
 * type will change to `{ [x: string]: unknown }`.
 *
 * Any arguments after `'--'` will not be parsed and will end up in `parsedArgs._`.
 *
 * Numeric-looking arguments will be returned as numbers unless `options.string`
 * or `options.boolean` is set for that argument name.
 *
 * @example
 * ```ts
 * import { parse } from "https://deno.land/std@$STD_VERSION/flags/mod.ts";
 * const parsedArgs = parse(Deno.args);
 * ```
 *
 * @example
 * ```ts
 * import { parse } from "https://deno.land/std@$STD_VERSION/flags/mod.ts";
 * const parsedArgs = parse(["--foo", "--bar=baz", "./quux.txt"]);
 * // parsedArgs: { foo: true, bar: "baz", _: ["./quux.txt"] }
 * ```
 */
function flagsParse(args, { "--": doubleDash = false, alias = {}, boolean = false, default: defaults = {}, stopEarly = false, string = [], collect = [], negatable = [], unknown = (i) => i, } = {}) {
    var _a;
    const aliases = {};
    const flags = {
        bools: {},
        strings: {},
        unknownFn: unknown,
        allBools: false,
        collect: {},
        negatable: {},
    };
    if (alias !== undefined) {
        for (const key in alias) {
            const val = getForce(alias, key);
            if (typeof val === "string") {
                aliases[key] = [val];
            }
            else {
                aliases[key] = val;
            }
            for (const alias of getForce(aliases, key)) {
                aliases[alias] = [key].concat(aliases[key].filter((y) => alias !== y));
            }
        }
    }
    if (boolean !== undefined) {
        if (typeof boolean === "boolean") {
            flags.allBools = !!boolean;
        }
        else {
            const booleanArgs = typeof boolean === "string"
                ? [boolean]
                : boolean;
            for (const key of booleanArgs.filter(Boolean)) {
                flags.bools[key] = true;
                const alias = get(aliases, key);
                if (alias) {
                    for (const al of alias) {
                        flags.bools[al] = true;
                    }
                }
            }
        }
    }
    if (string !== undefined) {
        const stringArgs = typeof string === "string"
            ? [string]
            : string;
        for (const key of stringArgs.filter(Boolean)) {
            flags.strings[key] = true;
            const alias = get(aliases, key);
            if (alias) {
                for (const al of alias) {
                    flags.strings[al] = true;
                }
            }
        }
    }
    if (collect !== undefined) {
        const collectArgs = typeof collect === "string"
            ? [collect]
            : collect;
        for (const key of collectArgs.filter(Boolean)) {
            flags.collect[key] = true;
            const alias = get(aliases, key);
            if (alias) {
                for (const al of alias) {
                    flags.collect[al] = true;
                }
            }
        }
    }
    if (negatable !== undefined) {
        const negatableArgs = typeof negatable === "string"
            ? [negatable]
            : negatable;
        for (const key of negatableArgs.filter(Boolean)) {
            flags.negatable[key] = true;
            const alias = get(aliases, key);
            if (alias) {
                for (const al of alias) {
                    flags.negatable[al] = true;
                }
            }
        }
    }
    const argv = { _: [] };
    function argDefined(key, arg) {
        return ((flags.allBools && /^--[^=]+$/.test(arg)) ||
            get(flags.bools, key) ||
            !!get(flags.strings, key) ||
            !!get(aliases, key));
    }
    function setKey(obj, name, value, collect = true) {
        let o = obj;
        const keys = name.split(".");
        keys.slice(0, -1).forEach(function (key) {
            if (get(o, key) === undefined) {
                o[key] = {};
            }
            o = get(o, key);
        });
        const key = keys[keys.length - 1];
        const collectable = collect && !!get(flags.collect, name);
        if (!collectable) {
            o[key] = value;
        }
        else if (get(o, key) === undefined) {
            o[key] = [value];
        }
        else if (Array.isArray(get(o, key))) {
            o[key].push(value);
        }
        else {
            o[key] = [get(o, key), value];
        }
    }
    function setArg(key, val, arg = undefined, collect) {
        if (arg && flags.unknownFn && !argDefined(key, arg)) {
            if (flags.unknownFn(arg, key, val) === false)
                return;
        }
        const value = !get(flags.strings, key) && isNumber(val) ? Number(val) : val;
        setKey(argv, key, value, collect);
        const alias = get(aliases, key);
        if (alias) {
            for (const x of alias) {
                setKey(argv, x, value, collect);
            }
        }
    }
    function aliasIsBoolean(key) {
        return getForce(aliases, key).some((x) => typeof get(flags.bools, x) === "boolean");
    }
    let notFlags = [];
    // all args after "--" are not parsed
    if (args.includes("--")) {
        notFlags = args.slice(args.indexOf("--") + 1);
        args = args.slice(0, args.indexOf("--"));
    }
    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (/^--.+=/.test(arg)) {
            const m = arg.match(/^--([^=]+)=(.*)$/s);
            assert(m !== null);
            const [, key, value] = m;
            if (flags.bools[key]) {
                const booleanValue = value !== "false";
                setArg(key, booleanValue, arg);
            }
            else {
                setArg(key, value, arg);
            }
        }
        else if (/^--no-.+/.test(arg) && get(flags.negatable, arg.replace(/^--no-/, ""))) {
            const m = arg.match(/^--no-(.+)/);
            assert(m !== null);
            setArg(m[1], false, arg, false);
        }
        else if (/^--.+/.test(arg)) {
            const m = arg.match(/^--(.+)/);
            assert(m !== null);
            const [, key] = m;
            const next = args[i + 1];
            if (next !== undefined &&
                !/^-/.test(next) &&
                !get(flags.bools, key) &&
                !flags.allBools &&
                (get(aliases, key) ? !aliasIsBoolean(key) : true)) {
                setArg(key, next, arg);
                i++;
            }
            else if (/^(true|false)$/.test(next)) {
                setArg(key, next === "true", arg);
                i++;
            }
            else {
                setArg(key, get(flags.strings, key) ? "" : true, arg);
            }
        }
        else if (/^-[^-]+/.test(arg)) {
            const letters = arg.slice(1, -1).split("");
            let broken = false;
            for (let j = 0; j < letters.length; j++) {
                const next = arg.slice(j + 2);
                if (next === "-") {
                    setArg(letters[j], next, arg);
                    continue;
                }
                if (/[A-Za-z]/.test(letters[j]) && /=/.test(next)) {
                    setArg(letters[j], next.split(/=(.+)/)[1], arg);
                    broken = true;
                    break;
                }
                if (/[A-Za-z]/.test(letters[j]) &&
                    /-?\d+(\.\d*)?(e-?\d+)?$/.test(next)) {
                    setArg(letters[j], next, arg);
                    broken = true;
                    break;
                }
                if (letters[j + 1] && letters[j + 1].match(/\W/)) {
                    setArg(letters[j], arg.slice(j + 2), arg);
                    broken = true;
                    break;
                }
                else {
                    setArg(letters[j], get(flags.strings, letters[j]) ? "" : true, arg);
                }
            }
            const [key] = arg.slice(-1);
            if (!broken && key !== "-") {
                if (args[i + 1] &&
                    !/^(-|--)[^-]/.test(args[i + 1]) &&
                    !get(flags.bools, key) &&
                    (get(aliases, key) ? !aliasIsBoolean(key) : true)) {
                    setArg(key, args[i + 1], arg);
                    i++;
                }
                else if (args[i + 1] && /^(true|false)$/.test(args[i + 1])) {
                    setArg(key, args[i + 1] === "true", arg);
                    i++;
                }
                else {
                    setArg(key, get(flags.strings, key) ? "" : true, arg);
                }
            }
        }
        else {
            if (!flags.unknownFn || flags.unknownFn(arg) !== false) {
                argv._.push(((_a = flags.strings["_"]) !== null && _a !== void 0 ? _a : !isNumber(arg)) ? arg : Number(arg));
            }
            if (stopEarly) {
                argv._.push(...args.slice(i + 1));
                break;
            }
        }
    }
    for (const [key, value] of Object.entries(defaults)) {
        if (!hasKey(argv, key.split("."))) {
            setKey(argv, key, value, false);
            if (aliases[key]) {
                for (const x of aliases[key]) {
                    setKey(argv, x, value, false);
                }
            }
        }
    }
    for (const key of Object.keys(flags.bools)) {
        if (!hasKey(argv, key.split("."))) {
            const value = get(flags.collect, key) ? [] : false;
            setKey(argv, key, value, false);
        }
    }
    for (const key of Object.keys(flags.strings)) {
        if (!hasKey(argv, key.split(".")) && get(flags.collect, key)) {
            setKey(argv, key, [], false);
        }
    }
    if (doubleDash) {
        argv["--"] = [];
        for (const key of notFlags) {
            argv["--"].push(key);
        }
    }
    else {
        for (const key of notFlags) {
            argv._.push(key);
        }
    }
    return argv;
}

globalThis.Luwak.cli = {
    flags: (options) => {
        return flagsParse(Deno.args, options)
    },
    args : () => {
        return Deno.args
    }
};

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
    res
}

globalThis.Luwak.wasm = WebAssembly

globalThis.Luwak.wasm.instantiateFetch = async (url) => {
    return await WebAssembly.instantiateStreaming(
        fetch(url),
    );
}

globalThis.Luwak.wasm.compileFetch = async (url) => {
    return await WebAssembly.compileStreaming(
        fetch(url),
    );
}