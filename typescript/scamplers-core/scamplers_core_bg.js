let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


function getFromExternrefTable0(idx) { return wasm.__wbindgen_export_0.get(idx); }

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getCachedStringFromWasm0(ptr, len) {
    if (ptr === 0) {
        return getFromExternrefTable0(len);
    } else {
        return getStringFromWasm0(ptr, len);
    }
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_0.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_5.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_0.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_0.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}
function __wbg_adapter_40(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h05b0a418da3bc948(arg0, arg1);
}

function __wbg_adapter_43(arg0, arg1, arg2) {
    wasm.closure123_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_424(arg0, arg1, arg2, arg3) {
    wasm.closure158_externref_shim(arg0, arg1, arg2, arg3);
}

/**
 * @enum {0 | 1 | 2 | 3}
 */
export const ComplianceCommitteeType = Object.freeze({
    Ibc: 0, "0": "Ibc",
    Irb: 1, "1": "Irb",
    Iacuc: 2, "2": "Iacuc",
    Unknown: 3, "3": "Unknown",
});
/**
 * @enum {0}
 */
export const InstitutionOrdinalColumn = Object.freeze({
    Name: 0, "0": "Name",
});
/**
 * @enum {0}
 */
export const LabOrdinalColumn = Object.freeze({
    Name: 0, "0": "Name",
});
/**
 * @enum {0 | 1}
 */
export const PersonOrdinalColumn = Object.freeze({
    Name: 0, "0": "Name",
    Email: 1, "1": "Email",
});
/**
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8}
 */
export const Species = Object.freeze({
    AmbystomaMexicanum: 0, "0": "AmbystomaMexicanum",
    CanisFamiliaris: 1, "1": "CanisFamiliaris",
    DrosophilaMelanogaster: 2, "2": "DrosophilaMelanogaster",
    GasterosteusAculeatus: 3, "3": "GasterosteusAculeatus",
    HomoSapiens: 4, "4": "HomoSapiens",
    MusMusculus: 5, "5": "MusMusculus",
    RattusNorvegicus: 6, "6": "RattusNorvegicus",
    SminthopsisCrassicaudata: 7, "7": "SminthopsisCrassicaudata",
    Unknown: 8, "8": "Unknown",
});
/**
 * @enum {0 | 1 | 2 | 3}
 */
export const UserRole = Object.freeze({
    AppAdmin: 0, "0": "AppAdmin",
    ComputationalStaff: 1, "1": "ComputationalStaff",
    BiologyStaff: 2, "2": "BiologyStaff",
    Unknown: 3, "3": "Unknown",
});

const __wbindgen_enum_RequestCredentials = ["omit", "same-origin", "include"];

const __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];

const ClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_client_free(ptr >>> 0, 1));

export class Client {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ClientFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_client_free(ptr, 0);
    }
    /**
     * @param {NewInstitution} data
     * @param {string | null} [api_key]
     * @returns {Promise<Institution>}
     */
    send_new_institution(data, api_key) {
        _assertClass(data, NewInstitution);
        var ptr0 = isLikeNone(api_key) ? 0 : passStringToWasm0(api_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.client_send_new_institution(this.__wbg_ptr, data.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @param {NewPerson} data
     * @param {string | null} [api_key]
     * @returns {Promise<Person>}
     */
    send_new_person(data, api_key) {
        _assertClass(data, NewPerson);
        var ptr0 = isLikeNone(api_key) ? 0 : passStringToWasm0(api_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.client_send_new_person(this.__wbg_ptr, data.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @param {NewLab} data
     * @param {string | null} [api_key]
     * @returns {Promise<Lab>}
     */
    send_new_lab(data, api_key) {
        _assertClass(data, NewLab);
        var ptr0 = isLikeNone(api_key) ? 0 : passStringToWasm0(api_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.client_send_new_lab(this.__wbg_ptr, data.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @param {string} backend_url
     * @param {string} token
     */
    constructor(backend_url, token) {
        const ptr0 = passStringToWasm0(backend_url, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(token, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.client_new(ptr0, len0, ptr1, len1);
        this.__wbg_ptr = ret >>> 0;
        ClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {NewPerson} data
     * @returns {Promise<CreatedUser>}
     */
    send_new_ms_login(data) {
        _assertClass(data, NewPerson);
        const ret = wasm.client_send_new_ms_login(this.__wbg_ptr, data.__wbg_ptr);
        return ret;
    }
}

const CreatedUserFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_createduser_free(ptr >>> 0, 1));

export class CreatedUser {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(CreatedUser.prototype);
        obj.__wbg_ptr = ptr;
        CreatedUserFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CreatedUserFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_createduser_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.createduser_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.createduser_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.createduser_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.createduser_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get orcid() {
        const ret = wasm.createduser_orcid(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {Institution}
     */
    get institution() {
        const ret = wasm.createduser_institution(this.__wbg_ptr);
        return Institution.__wrap(ret);
    }
    /**
     * @returns {any[]}
     */
    get roles() {
        const ret = wasm.createduser_roles(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string}
     */
    get api_key() {
        const ret = wasm.createduser_api_key(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const InstitutionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institution_free(ptr >>> 0, 1));

export class Institution {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Institution.prototype);
        obj.__wbg_ptr = ptr;
        InstitutionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institution_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.institution_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.institution_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.institution_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const InstitutionOrderingFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institutionordering_free(ptr >>> 0, 1));

export class InstitutionOrdering {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(InstitutionOrdering.prototype);
        obj.__wbg_ptr = ptr;
        InstitutionOrderingFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof InstitutionOrdering)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    toJSON() {
        return {
            column: this.column,
            descending: this.descending,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionOrderingFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institutionordering_free(ptr, 0);
    }
    /**
     * @returns {InstitutionOrdinalColumn}
     */
    get column() {
        const ret = wasm.__wbg_get_institutionordering_column(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {InstitutionOrdinalColumn} arg0
     */
    set column(arg0) {
        wasm.__wbg_set_institutionordering_column(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get descending() {
        const ret = wasm.__wbg_get_institutionordering_descending(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set descending(arg0) {
        wasm.__wbg_set_institutionordering_descending(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {InstitutionOrderingBuilder}
     */
    static new() {
        const ret = wasm.institutionordering_new();
        return InstitutionOrderingBuilder.__wrap(ret);
    }
}

const InstitutionOrderingBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institutionorderingbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`InstitutionOrdering`](struct.InstitutionOrdering.html).
 */
export class InstitutionOrderingBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(InstitutionOrderingBuilder.prototype);
        obj.__wbg_ptr = ptr;
        InstitutionOrderingBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionOrderingBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institutionorderingbuilder_free(ptr, 0);
    }
    /**
     * @param {InstitutionOrdinalColumn} value
     * @returns {InstitutionOrderingBuilder}
     */
    column(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.institutionorderingbuilder_column(ptr, value);
        return InstitutionOrderingBuilder.__wrap(ret);
    }
    /**
     * @param {boolean} value
     * @returns {InstitutionOrderingBuilder}
     */
    descending(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.institutionorderingbuilder_descending(ptr, value);
        return InstitutionOrderingBuilder.__wrap(ret);
    }
    /**
     * Builds a new `InstitutionOrdering`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {InstitutionOrdering}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.institutionorderingbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return InstitutionOrdering.__wrap(ret[0]);
    }
}

const InstitutionOrderingErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institutionorderingerror_free(ptr >>> 0, 1));

export class InstitutionOrderingError {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionOrderingErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institutionorderingerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.institutionorderingerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const InstitutionQueryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institutionquery_free(ptr >>> 0, 1));

export class InstitutionQuery {

    toJSON() {
        return {
            ids: this.ids,
            name: this.name,
            order_by: this.order_by,
            pagination: this.pagination,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionQueryFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institutionquery_free(ptr, 0);
    }
    /**
     * @returns {string[]}
     */
    get ids() {
        const ret = wasm.__wbg_get_institutionquery_ids(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set ids(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_institutionquery_ids(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_institutionquery_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set name(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_institutionquery_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {InstitutionOrdering[]}
     */
    get order_by() {
        const ret = wasm.__wbg_get_institutionquery_order_by(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {InstitutionOrdering[]} arg0
     */
    set order_by(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_institutionquery_order_by(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Pagination}
     */
    get pagination() {
        const ret = wasm.__wbg_get_institutionquery_pagination(this.__wbg_ptr);
        return Pagination.__wrap(ret);
    }
    /**
     * @param {Pagination} arg0
     */
    set pagination(arg0) {
        _assertClass(arg0, Pagination);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_institutionquery_pagination(this.__wbg_ptr, ptr0);
    }
    constructor() {
        const ret = wasm.institutionquery_new();
        this.__wbg_ptr = ret >>> 0;
        InstitutionQueryFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const InstitutionReferenceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institutionreference_free(ptr >>> 0, 1));

export class InstitutionReference {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionReferenceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institutionreference_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.institutionreference_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.institutionreference_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const InstitutionSummaryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_institutionsummary_free(ptr >>> 0, 1));

export class InstitutionSummary {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        InstitutionSummaryFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_institutionsummary_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.institutionsummary_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.institutionsummary_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.institutionsummary_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const LabFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_lab_free(ptr >>> 0, 1));

export class Lab {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Lab.prototype);
        obj.__wbg_ptr = ptr;
        LabFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_lab_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.lab_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.lab_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.lab_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get delivery_dir() {
        const ret = wasm.lab_delivery_dir(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {PersonSummary}
     */
    get pi() {
        const ret = wasm.lab_pi(this.__wbg_ptr);
        return PersonSummary.__wrap(ret);
    }
    /**
     * @returns {PersonSummary[]}
     */
    get members() {
        const ret = wasm.lab_members(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}

const LabDataFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labdata_free(ptr >>> 0, 1));

export class LabData {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabDataFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labdata_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.labdata_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.labdata_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.labdata_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get delivery_dir() {
        const ret = wasm.labdata_delivery_dir(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {PersonSummary}
     */
    get pi() {
        const ret = wasm.labdata_pi(this.__wbg_ptr);
        return PersonSummary.__wrap(ret);
    }
}

const LabOrderingFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labordering_free(ptr >>> 0, 1));

export class LabOrdering {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabOrdering.prototype);
        obj.__wbg_ptr = ptr;
        LabOrderingFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof LabOrdering)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    toJSON() {
        return {
            column: this.column,
            descending: this.descending,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabOrderingFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labordering_free(ptr, 0);
    }
    /**
     * @returns {LabOrdinalColumn}
     */
    get column() {
        const ret = wasm.__wbg_get_labordering_column(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {LabOrdinalColumn} arg0
     */
    set column(arg0) {
        wasm.__wbg_set_labordering_column(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get descending() {
        const ret = wasm.__wbg_get_labordering_descending(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set descending(arg0) {
        wasm.__wbg_set_labordering_descending(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {LabOrderingBuilder}
     */
    static new() {
        const ret = wasm.labordering_new();
        return LabOrderingBuilder.__wrap(ret);
    }
}

const LabOrderingBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_laborderingbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`LabOrdering`](struct.LabOrdering.html).
 */
export class LabOrderingBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabOrderingBuilder.prototype);
        obj.__wbg_ptr = ptr;
        LabOrderingBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabOrderingBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_laborderingbuilder_free(ptr, 0);
    }
    /**
     * @param {LabOrdinalColumn} value
     * @returns {LabOrderingBuilder}
     */
    column(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.laborderingbuilder_column(ptr, value);
        return LabOrderingBuilder.__wrap(ret);
    }
    /**
     * @param {boolean} value
     * @returns {LabOrderingBuilder}
     */
    descending(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.laborderingbuilder_descending(ptr, value);
        return LabOrderingBuilder.__wrap(ret);
    }
    /**
     * Builds a new `LabOrdering`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {LabOrdering}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.laborderingbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return LabOrdering.__wrap(ret[0]);
    }
}

const LabOrderingErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_laborderingerror_free(ptr >>> 0, 1));

export class LabOrderingError {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabOrderingErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_laborderingerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.laborderingerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const LabQueryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labquery_free(ptr >>> 0, 1));

export class LabQuery {

    toJSON() {
        return {
            ids: this.ids,
            name: this.name,
            order_by: this.order_by,
            pagination: this.pagination,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabQueryFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labquery_free(ptr, 0);
    }
    /**
     * @returns {string[]}
     */
    get ids() {
        const ret = wasm.__wbg_get_labquery_ids(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set ids(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labquery_ids(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_labquery_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set name(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labquery_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {LabOrdering[]}
     */
    get order_by() {
        const ret = wasm.__wbg_get_labquery_order_by(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {LabOrdering[]} arg0
     */
    set order_by(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labquery_order_by(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Pagination}
     */
    get pagination() {
        const ret = wasm.__wbg_get_labquery_pagination(this.__wbg_ptr);
        return Pagination.__wrap(ret);
    }
    /**
     * @param {Pagination} arg0
     */
    set pagination(arg0) {
        _assertClass(arg0, Pagination);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_labquery_pagination(this.__wbg_ptr, ptr0);
    }
    constructor() {
        const ret = wasm.labquery_new();
        this.__wbg_ptr = ret >>> 0;
        LabQueryFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const LabReferenceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labreference_free(ptr >>> 0, 1));

export class LabReference {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabReferenceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labreference_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.labreference_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.labreference_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const LabSummaryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labsummary_free(ptr >>> 0, 1));

export class LabSummary {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabSummaryFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labsummary_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.labsummary_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.labsummary_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.labsummary_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get delivery_dir() {
        const ret = wasm.labsummary_delivery_dir(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const LabUpdateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labupdate_free(ptr >>> 0, 1));

export class LabUpdate {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabUpdate.prototype);
        obj.__wbg_ptr = ptr;
        LabUpdateFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            id: this.id,
            name: this.name,
            pi_id: this.pi_id,
            delivery_dir: this.delivery_dir,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabUpdateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labupdate_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.__wbg_get_labupdate_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labupdate_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_labupdate_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set name(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labupdate_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get pi_id() {
        const ret = wasm.__wbg_get_labupdate_pi_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set pi_id(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labupdate_pi_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get delivery_dir() {
        const ret = wasm.__wbg_get_labupdate_delivery_dir(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set delivery_dir(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labupdate_delivery_dir(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {LabUpdateBuilder}
     */
    static new() {
        const ret = wasm.labupdate_new();
        return LabUpdateBuilder.__wrap(ret);
    }
}

const LabUpdateBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labupdatebuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`LabUpdate`](struct.LabUpdate.html).
 */
export class LabUpdateBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabUpdateBuilder.prototype);
        obj.__wbg_ptr = ptr;
        LabUpdateBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabUpdateBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labupdatebuilder_free(ptr, 0);
    }
    /**
     * @param {string} value
     * @returns {LabUpdateBuilder}
     */
    id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.labupdatebuilder_id(ptr, ptr0, len0);
        return LabUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {LabUpdateBuilder}
     */
    name(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.labupdatebuilder_name(ptr, ptr0, len0);
        return LabUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {LabUpdateBuilder}
     */
    pi_id(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.labupdatebuilder_pi_id(ptr, ptr0, len0);
        return LabUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {LabUpdateBuilder}
     */
    delivery_dir(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.labupdatebuilder_delivery_dir(ptr, ptr0, len0);
        return LabUpdateBuilder.__wrap(ret);
    }
    /**
     * Builds a new `LabUpdate`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {LabUpdate}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.labupdatebuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return LabUpdate.__wrap(ret[0]);
    }
}

const LabUpdateErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labupdateerror_free(ptr >>> 0, 1));

export class LabUpdateError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabUpdateError.prototype);
        obj.__wbg_ptr = ptr;
        LabUpdateErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabUpdateErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labupdateerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.labupdateerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const LabUpdateWithMembersFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labupdatewithmembers_free(ptr >>> 0, 1));

export class LabUpdateWithMembers {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabUpdateWithMembers.prototype);
        obj.__wbg_ptr = ptr;
        LabUpdateWithMembersFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            update: this.update,
            add_members: this.add_members,
            remove_members: this.remove_members,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabUpdateWithMembersFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labupdatewithmembers_free(ptr, 0);
    }
    /**
     * @returns {LabUpdate}
     */
    get update() {
        const ret = wasm.__wbg_get_labupdatewithmembers_update(this.__wbg_ptr);
        return LabUpdate.__wrap(ret);
    }
    /**
     * @param {LabUpdate} arg0
     */
    set update(arg0) {
        _assertClass(arg0, LabUpdate);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_labupdatewithmembers_update(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {string[]}
     */
    get add_members() {
        const ret = wasm.__wbg_get_labupdatewithmembers_add_members(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set add_members(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labupdatewithmembers_add_members(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string[]}
     */
    get remove_members() {
        const ret = wasm.__wbg_get_labupdatewithmembers_remove_members(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set remove_members(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_labupdatewithmembers_remove_members(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {LabUpdateWithMembersBuilder}
     */
    static new() {
        const ret = wasm.labupdatewithmembers_new();
        return LabUpdateWithMembersBuilder.__wrap(ret);
    }
}

const LabUpdateWithMembersBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labupdatewithmembersbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`LabUpdateWithMembers`](struct.LabUpdateWithMembers.html).
 */
export class LabUpdateWithMembersBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(LabUpdateWithMembersBuilder.prototype);
        obj.__wbg_ptr = ptr;
        LabUpdateWithMembersBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabUpdateWithMembersBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labupdatewithmembersbuilder_free(ptr, 0);
    }
    /**
     * @param {LabUpdate} value
     * @returns {LabUpdateWithMembersBuilder}
     */
    update(value) {
        const ptr = this.__destroy_into_raw();
        _assertClass(value, LabUpdate);
        var ptr0 = value.__destroy_into_raw();
        const ret = wasm.labupdatewithmembersbuilder_update(ptr, ptr0);
        return LabUpdateWithMembersBuilder.__wrap(ret);
    }
    /**
     * @param {string[]} value
     * @returns {LabUpdateWithMembersBuilder}
     */
    add_members(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.labupdatewithmembersbuilder_add_members(ptr, ptr0, len0);
        return LabUpdateWithMembersBuilder.__wrap(ret);
    }
    /**
     * @param {string[]} value
     * @returns {LabUpdateWithMembersBuilder}
     */
    remove_members(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.labupdatewithmembersbuilder_remove_members(ptr, ptr0, len0);
        return LabUpdateWithMembersBuilder.__wrap(ret);
    }
    /**
     * Builds a new `LabUpdateWithMembers`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {LabUpdateWithMembers}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.labupdatewithmembersbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return LabUpdateWithMembers.__wrap(ret[0]);
    }
}

const LabUpdateWithMembersErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_labupdatewithmemberserror_free(ptr >>> 0, 1));

export class LabUpdateWithMembersError {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        LabUpdateWithMembersErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_labupdatewithmemberserror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.labupdatewithmemberserror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const NewCommitteeApprovalFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newcommitteeapproval_free(ptr >>> 0, 1));

export class NewCommitteeApproval {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewCommitteeApproval.prototype);
        obj.__wbg_ptr = ptr;
        NewCommitteeApprovalFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof NewCommitteeApproval)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    toJSON() {
        return {
            sample_id: this.sample_id,
            institution_id: this.institution_id,
            committee_type: this.committee_type,
            compliance_identifier: this.compliance_identifier,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewCommitteeApprovalFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newcommitteeapproval_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get sample_id() {
        const ret = wasm.__wbg_get_newcommitteeapproval_sample_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set sample_id(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newcommitteeapproval_sample_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get institution_id() {
        const ret = wasm.__wbg_get_newcommitteeapproval_institution_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set institution_id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newcommitteeapproval_institution_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {ComplianceCommitteeType}
     */
    get committee_type() {
        const ret = wasm.__wbg_get_newcommitteeapproval_committee_type(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {ComplianceCommitteeType} arg0
     */
    set committee_type(arg0) {
        wasm.__wbg_set_newcommitteeapproval_committee_type(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string}
     */
    get compliance_identifier() {
        const ret = wasm.__wbg_get_newcommitteeapproval_compliance_identifier(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set compliance_identifier(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newcommitteeapproval_compliance_identifier(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {NewCommitteeApprovalBuilder}
     */
    static new() {
        const ret = wasm.newcommitteeapproval_new();
        return NewCommitteeApprovalBuilder.__wrap(ret);
    }
}

const NewCommitteeApprovalBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newcommitteeapprovalbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`NewCommitteeApproval`](struct.NewCommitteeApproval.html).
 */
export class NewCommitteeApprovalBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewCommitteeApprovalBuilder.prototype);
        obj.__wbg_ptr = ptr;
        NewCommitteeApprovalBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewCommitteeApprovalBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newcommitteeapprovalbuilder_free(ptr, 0);
    }
    /**
     * @param {string | null} [value]
     * @returns {NewCommitteeApprovalBuilder}
     */
    sample_id(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.newcommitteeapprovalbuilder_sample_id(ptr, ptr0, len0);
        return NewCommitteeApprovalBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewCommitteeApprovalBuilder}
     */
    institution_id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newcommitteeapprovalbuilder_institution_id(ptr, ptr0, len0);
        return NewCommitteeApprovalBuilder.__wrap(ret);
    }
    /**
     * @param {ComplianceCommitteeType} value
     * @returns {NewCommitteeApprovalBuilder}
     */
    committee_type(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.newcommitteeapprovalbuilder_committee_type(ptr, value);
        return NewCommitteeApprovalBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewCommitteeApprovalBuilder}
     */
    compliance_identifier(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newcommitteeapprovalbuilder_compliance_identifier(ptr, ptr0, len0);
        return NewCommitteeApprovalBuilder.__wrap(ret);
    }
    /**
     * Builds a new `NewCommitteeApproval`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {NewCommitteeApproval}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.newcommitteeapprovalbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return NewCommitteeApproval.__wrap(ret[0]);
    }
}

const NewCommitteeApprovalErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newcommitteeapprovalerror_free(ptr >>> 0, 1));

export class NewCommitteeApprovalError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewCommitteeApprovalError.prototype);
        obj.__wbg_ptr = ptr;
        NewCommitteeApprovalErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewCommitteeApprovalErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newcommitteeapprovalerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.newcommitteeapprovalerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const NewInstitutionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newinstitution_free(ptr >>> 0, 1));

export class NewInstitution {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewInstitution.prototype);
        obj.__wbg_ptr = ptr;
        NewInstitutionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            id: this.id,
            name: this.name,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewInstitutionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newinstitution_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.__wbg_get_newinstitution_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newinstitution_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_newinstitution_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set name(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newinstitution_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {NewInstitutionBuilder}
     */
    static new() {
        const ret = wasm.newinstitution_new();
        return NewInstitutionBuilder.__wrap(ret);
    }
}

const NewInstitutionBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newinstitutionbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`NewInstitution`](struct.NewInstitution.html).
 */
export class NewInstitutionBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewInstitutionBuilder.prototype);
        obj.__wbg_ptr = ptr;
        NewInstitutionBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewInstitutionBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newinstitutionbuilder_free(ptr, 0);
    }
    /**
     * @param {string} value
     * @returns {NewInstitutionBuilder}
     */
    id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newinstitutionbuilder_id(ptr, ptr0, len0);
        return NewInstitutionBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewInstitutionBuilder}
     */
    name(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newinstitutionbuilder_name(ptr, ptr0, len0);
        return NewInstitutionBuilder.__wrap(ret);
    }
    /**
     * Builds a new `NewInstitution`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {NewInstitution}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.newinstitutionbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return NewInstitution.__wrap(ret[0]);
    }
}

const NewInstitutionErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newinstitutionerror_free(ptr >>> 0, 1));

export class NewInstitutionError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewInstitutionError.prototype);
        obj.__wbg_ptr = ptr;
        NewInstitutionErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewInstitutionErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newinstitutionerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.newinstitutionerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const NewLabFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newlab_free(ptr >>> 0, 1));

export class NewLab {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewLab.prototype);
        obj.__wbg_ptr = ptr;
        NewLabFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            name: this.name,
            pi_id: this.pi_id,
            delivery_dir: this.delivery_dir,
            member_ids: this.member_ids,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewLabFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newlab_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_newlab_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set name(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newlab_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get pi_id() {
        const ret = wasm.__wbg_get_newlab_pi_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set pi_id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newlab_pi_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get delivery_dir() {
        const ret = wasm.__wbg_get_newlab_delivery_dir(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set delivery_dir(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newlab_delivery_dir(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string[]}
     */
    get member_ids() {
        const ret = wasm.__wbg_get_newlab_member_ids(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set member_ids(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newlab_member_ids(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {NewLabBuilder}
     */
    static new() {
        const ret = wasm.newlab_new();
        return NewLabBuilder.__wrap(ret);
    }
}

const NewLabBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newlabbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`NewLab`](struct.NewLab.html).
 */
export class NewLabBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewLabBuilder.prototype);
        obj.__wbg_ptr = ptr;
        NewLabBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewLabBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newlabbuilder_free(ptr, 0);
    }
    /**
     * @param {string} value
     * @returns {NewLabBuilder}
     */
    name(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newlabbuilder_name(ptr, ptr0, len0);
        return NewLabBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewLabBuilder}
     */
    pi_id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newlabbuilder_pi_id(ptr, ptr0, len0);
        return NewLabBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewLabBuilder}
     */
    delivery_dir(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newlabbuilder_delivery_dir(ptr, ptr0, len0);
        return NewLabBuilder.__wrap(ret);
    }
    /**
     * @param {string[]} value
     * @returns {NewLabBuilder}
     */
    member_ids(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newlabbuilder_member_ids(ptr, ptr0, len0);
        return NewLabBuilder.__wrap(ret);
    }
    /**
     * Builds a new `NewLab`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {NewLab}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.newlabbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return NewLab.__wrap(ret[0]);
    }
}

const NewLabErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newlaberror_free(ptr >>> 0, 1));

export class NewLabError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewLabError.prototype);
        obj.__wbg_ptr = ptr;
        NewLabErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewLabErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newlaberror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.newlaberror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const NewPersonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newperson_free(ptr >>> 0, 1));

export class NewPerson {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewPerson.prototype);
        obj.__wbg_ptr = ptr;
        NewPersonFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            name: this.name,
            email: this.email,
            orcid: this.orcid,
            institution_id: this.institution_id,
            ms_user_id: this.ms_user_id,
            roles: this.roles,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewPersonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newperson_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_newperson_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set name(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newperson_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.__wbg_get_newperson_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set email(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newperson_email(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get orcid() {
        const ret = wasm.__wbg_get_newperson_orcid(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set orcid(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newperson_orcid(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get institution_id() {
        const ret = wasm.__wbg_get_newperson_institution_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set institution_id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newperson_institution_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get ms_user_id() {
        const ret = wasm.__wbg_get_newperson_ms_user_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set ms_user_id(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newperson_ms_user_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {any[]}
     */
    get roles() {
        const ret = wasm.__wbg_get_newperson_roles(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {any[]} arg0
     */
    set roles(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newperson_roles(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {NewPersonBuilder}
     */
    static new() {
        const ret = wasm.newperson_new();
        return NewPersonBuilder.__wrap(ret);
    }
}

const NewPersonBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newpersonbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`NewPerson`](struct.NewPerson.html).
 */
export class NewPersonBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewPersonBuilder.prototype);
        obj.__wbg_ptr = ptr;
        NewPersonBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewPersonBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newpersonbuilder_free(ptr, 0);
    }
    /**
     * @param {string} value
     * @returns {NewPersonBuilder}
     */
    name(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newpersonbuilder_name(ptr, ptr0, len0);
        return NewPersonBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewPersonBuilder}
     */
    email(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newpersonbuilder_email(ptr, ptr0, len0);
        return NewPersonBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {NewPersonBuilder}
     */
    orcid(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.newpersonbuilder_orcid(ptr, ptr0, len0);
        return NewPersonBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewPersonBuilder}
     */
    institution_id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newpersonbuilder_institution_id(ptr, ptr0, len0);
        return NewPersonBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {NewPersonBuilder}
     */
    ms_user_id(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.newpersonbuilder_ms_user_id(ptr, ptr0, len0);
        return NewPersonBuilder.__wrap(ret);
    }
    /**
     * @param {any[]} value
     * @returns {NewPersonBuilder}
     */
    roles(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newpersonbuilder_roles(ptr, ptr0, len0);
        return NewPersonBuilder.__wrap(ret);
    }
    /**
     * Builds a new `NewPerson`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {NewPerson}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.newpersonbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return NewPerson.__wrap(ret[0]);
    }
}

const NewPersonErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newpersonerror_free(ptr >>> 0, 1));

export class NewPersonError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewPersonError.prototype);
        obj.__wbg_ptr = ptr;
        NewPersonErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewPersonErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newpersonerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.newpersonerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const NewSampleMetadataFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newsamplemetadata_free(ptr >>> 0, 1));

export class NewSampleMetadata {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewSampleMetadata.prototype);
        obj.__wbg_ptr = ptr;
        NewSampleMetadataFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            name: this.name,
            submitted_by: this.submitted_by,
            lab_id: this.lab_id,
            species: this.species,
            tissue: this.tissue,
            committee_approvals: this.committee_approvals,
            notes: this.notes,
            returned_by: this.returned_by,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewSampleMetadataFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newsamplemetadata_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_newsamplemetadata_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set name(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newcommitteeapproval_compliance_identifier(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get submitted_by() {
        const ret = wasm.__wbg_get_newsamplemetadata_submitted_by(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set submitted_by(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_submitted_by(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get lab_id() {
        const ret = wasm.__wbg_get_newsamplemetadata_lab_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set lab_id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_lab_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {any[]}
     */
    get species() {
        const ret = wasm.__wbg_get_newsamplemetadata_species(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {any[]} arg0
     */
    set species(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_species(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get tissue() {
        const ret = wasm.__wbg_get_newsamplemetadata_tissue(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set tissue(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_tissue(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {NewCommitteeApproval[]}
     */
    get committee_approvals() {
        const ret = wasm.__wbg_get_newsamplemetadata_committee_approvals(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {NewCommitteeApproval[]} arg0
     */
    set committee_approvals(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_committee_approvals(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string[] | undefined}
     */
    get notes() {
        const ret = wasm.__wbg_get_newsamplemetadata_notes(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        }
        return v1;
    }
    /**
     * @param {string[] | null} [arg0]
     */
    set notes(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_notes(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get returned_by() {
        const ret = wasm.__wbg_get_newsamplemetadata_returned_by(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set returned_by(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_newsamplemetadata_returned_by(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {NewSampleMetadataBuilder}
     */
    static new() {
        const ret = wasm.newsamplemetadata_new();
        return NewSampleMetadataBuilder.__wrap(ret);
    }
}

const NewSampleMetadataBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newsamplemetadatabuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`NewSampleMetadata`](struct.NewSampleMetadata.html).
 */
export class NewSampleMetadataBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewSampleMetadataBuilder.prototype);
        obj.__wbg_ptr = ptr;
        NewSampleMetadataBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewSampleMetadataBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newsamplemetadatabuilder_free(ptr, 0);
    }
    /**
     * @param {string} value
     * @returns {NewSampleMetadataBuilder}
     */
    name(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_name(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewSampleMetadataBuilder}
     */
    submitted_by(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_submitted_by(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewSampleMetadataBuilder}
     */
    lab_id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_lab_id(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {any[]} value
     * @returns {NewSampleMetadataBuilder}
     */
    species(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_species(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {string} value
     * @returns {NewSampleMetadataBuilder}
     */
    tissue(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_tissue(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {NewCommitteeApproval[]} value
     * @returns {NewSampleMetadataBuilder}
     */
    committee_approvals(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_committee_approvals(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {string[] | null} [value]
     * @returns {NewSampleMetadataBuilder}
     */
    notes(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_notes(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {NewSampleMetadataBuilder}
     */
    returned_by(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.newsamplemetadatabuilder_returned_by(ptr, ptr0, len0);
        return NewSampleMetadataBuilder.__wrap(ret);
    }
    /**
     * Builds a new `NewSampleMetadata`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {NewSampleMetadata}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.newsamplemetadatabuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return NewSampleMetadata.__wrap(ret[0]);
    }
}

const NewSampleMetadataErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_newsamplemetadataerror_free(ptr >>> 0, 1));

export class NewSampleMetadataError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(NewSampleMetadataError.prototype);
        obj.__wbg_ptr = ptr;
        NewSampleMetadataErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NewSampleMetadataErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_newsamplemetadataerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.newsamplemetadataerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const PaginationFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pagination_free(ptr >>> 0, 1));

export class Pagination {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Pagination.prototype);
        obj.__wbg_ptr = ptr;
        PaginationFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            limit: this.limit,
            offset: this.offset,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PaginationFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pagination_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get limit() {
        const ret = wasm.__wbg_get_pagination_limit(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set limit(arg0) {
        wasm.__wbg_set_pagination_limit(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get offset() {
        const ret = wasm.__wbg_get_pagination_offset(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set offset(arg0) {
        wasm.__wbg_set_pagination_offset(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} limit
     * @param {bigint} offset
     */
    constructor(limit, offset) {
        const ret = wasm.pagination_new(limit, offset);
        this.__wbg_ptr = ret >>> 0;
        PaginationFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const PersonFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_person_free(ptr >>> 0, 1));

export class Person {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Person.prototype);
        obj.__wbg_ptr = ptr;
        PersonFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_person_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.person_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.person_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.person_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.person_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get orcid() {
        const ret = wasm.person_orcid(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {Institution}
     */
    get institution() {
        const ret = wasm.person_institution(this.__wbg_ptr);
        return Institution.__wrap(ret);
    }
    /**
     * @returns {any[]}
     */
    get roles() {
        const ret = wasm.person_roles(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
}

const PersonDataFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_persondata_free(ptr >>> 0, 1));

export class PersonData {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonDataFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_persondata_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.persondata_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.persondata_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.persondata_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.persondata_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get orcid() {
        const ret = wasm.persondata_orcid(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {Institution}
     */
    get institution() {
        const ret = wasm.persondata_institution(this.__wbg_ptr);
        return Institution.__wrap(ret);
    }
}

const PersonDataUpdateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_persondataupdate_free(ptr >>> 0, 1));

export class PersonDataUpdate {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonDataUpdate.prototype);
        obj.__wbg_ptr = ptr;
        PersonDataUpdateFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            id: this.id,
            name: this.name,
            email: this.email,
            ms_user_id: this.ms_user_id,
            orcid: this.orcid,
            institution_id: this.institution_id,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonDataUpdateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_persondataupdate_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.__wbg_get_persondataupdate_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string} arg0
     */
    set id(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_persondataupdate_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_persondataupdate_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set name(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_persondataupdate_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.__wbg_get_persondataupdate_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set email(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_persondataupdate_email(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get ms_user_id() {
        const ret = wasm.__wbg_get_persondataupdate_ms_user_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set ms_user_id(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_persondataupdate_ms_user_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get orcid() {
        const ret = wasm.__wbg_get_persondataupdate_orcid(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set orcid(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_persondataupdate_orcid(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get institution_id() {
        const ret = wasm.__wbg_get_persondataupdate_institution_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set institution_id(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_persondataupdate_institution_id(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {PersonDataUpdateBuilder}
     */
    static new() {
        const ret = wasm.persondataupdate_new();
        return PersonDataUpdateBuilder.__wrap(ret);
    }
}

const PersonDataUpdateBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_persondataupdatebuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`PersonDataUpdate`](struct.PersonDataUpdate.html).
 */
export class PersonDataUpdateBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonDataUpdateBuilder.prototype);
        obj.__wbg_ptr = ptr;
        PersonDataUpdateBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonDataUpdateBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_persondataupdatebuilder_free(ptr, 0);
    }
    /**
     * @param {string} value
     * @returns {PersonDataUpdateBuilder}
     */
    id(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.persondataupdatebuilder_id(ptr, ptr0, len0);
        return PersonDataUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {PersonDataUpdateBuilder}
     */
    name(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.persondataupdatebuilder_name(ptr, ptr0, len0);
        return PersonDataUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {PersonDataUpdateBuilder}
     */
    email(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.persondataupdatebuilder_email(ptr, ptr0, len0);
        return PersonDataUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {PersonDataUpdateBuilder}
     */
    ms_user_id(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.persondataupdatebuilder_ms_user_id(ptr, ptr0, len0);
        return PersonDataUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {PersonDataUpdateBuilder}
     */
    orcid(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.persondataupdatebuilder_orcid(ptr, ptr0, len0);
        return PersonDataUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {string | null} [value]
     * @returns {PersonDataUpdateBuilder}
     */
    institution_id(value) {
        const ptr = this.__destroy_into_raw();
        var ptr0 = isLikeNone(value) ? 0 : passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.persondataupdatebuilder_institution_id(ptr, ptr0, len0);
        return PersonDataUpdateBuilder.__wrap(ret);
    }
    /**
     * Builds a new `PersonDataUpdate`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {PersonDataUpdate}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.persondataupdatebuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return PersonDataUpdate.__wrap(ret[0]);
    }
}

const PersonDataUpdateErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_persondataupdateerror_free(ptr >>> 0, 1));

export class PersonDataUpdateError {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonDataUpdateErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_persondataupdateerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.persondataupdateerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const PersonOrderingFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personordering_free(ptr >>> 0, 1));

export class PersonOrdering {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonOrdering.prototype);
        obj.__wbg_ptr = ptr;
        PersonOrderingFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    static __unwrap(jsValue) {
        if (!(jsValue instanceof PersonOrdering)) {
            return 0;
        }
        return jsValue.__destroy_into_raw();
    }

    toJSON() {
        return {
            column: this.column,
            descending: this.descending,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonOrderingFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personordering_free(ptr, 0);
    }
    /**
     * @returns {PersonOrdinalColumn}
     */
    get column() {
        const ret = wasm.__wbg_get_personordering_column(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {PersonOrdinalColumn} arg0
     */
    set column(arg0) {
        wasm.__wbg_set_personordering_column(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get descending() {
        const ret = wasm.__wbg_get_personordering_descending(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set descending(arg0) {
        wasm.__wbg_set_personordering_descending(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {PersonOrderingBuilder}
     */
    static new() {
        const ret = wasm.personordering_new();
        return PersonOrderingBuilder.__wrap(ret);
    }
}

const PersonOrderingBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personorderingbuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`PersonOrdering`](struct.PersonOrdering.html).
 */
export class PersonOrderingBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonOrderingBuilder.prototype);
        obj.__wbg_ptr = ptr;
        PersonOrderingBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonOrderingBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personorderingbuilder_free(ptr, 0);
    }
    /**
     * @param {PersonOrdinalColumn} value
     * @returns {PersonOrderingBuilder}
     */
    column(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.personorderingbuilder_column(ptr, value);
        return PersonOrderingBuilder.__wrap(ret);
    }
    /**
     * @param {boolean} value
     * @returns {PersonOrderingBuilder}
     */
    descending(value) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.personorderingbuilder_descending(ptr, value);
        return PersonOrderingBuilder.__wrap(ret);
    }
    /**
     * Builds a new `PersonOrdering`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {PersonOrdering}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.personorderingbuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return PersonOrdering.__wrap(ret[0]);
    }
}

const PersonOrderingErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personorderingerror_free(ptr >>> 0, 1));

export class PersonOrderingError {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonOrderingErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personorderingerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.personorderingerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const PersonQueryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personquery_free(ptr >>> 0, 1));

export class PersonQuery {

    toJSON() {
        return {
            ids: this.ids,
            name: this.name,
            email: this.email,
            order_by: this.order_by,
            pagination: this.pagination,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonQueryFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personquery_free(ptr, 0);
    }
    /**
     * @returns {string[]}
     */
    get ids() {
        const ret = wasm.__wbg_get_personquery_ids(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set ids(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_personquery_ids(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.__wbg_get_personquery_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set name(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_personquery_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.__wbg_get_personquery_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @param {string | null} [arg0]
     */
    set email(arg0) {
        var ptr0 = isLikeNone(arg0) ? 0 : passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_personquery_email(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {PersonOrdering[]}
     */
    get order_by() {
        const ret = wasm.__wbg_get_personquery_order_by(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {PersonOrdering[]} arg0
     */
    set order_by(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_personquery_order_by(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Pagination}
     */
    get pagination() {
        const ret = wasm.__wbg_get_personquery_pagination(this.__wbg_ptr);
        return Pagination.__wrap(ret);
    }
    /**
     * @param {Pagination} arg0
     */
    set pagination(arg0) {
        _assertClass(arg0, Pagination);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_personquery_pagination(this.__wbg_ptr, ptr0);
    }
    constructor() {
        const ret = wasm.personquery_new();
        this.__wbg_ptr = ret >>> 0;
        PersonQueryFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}

const PersonReferenceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personreference_free(ptr >>> 0, 1));

export class PersonReference {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonReferenceFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personreference_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.personreference_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.personreference_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const PersonSummaryFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personsummary_free(ptr >>> 0, 1));

export class PersonSummary {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonSummary.prototype);
        obj.__wbg_ptr = ptr;
        PersonSummaryFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonSummaryFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personsummary_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get id() {
        const ret = wasm.personsummary_id(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get link() {
        const ret = wasm.personsummary_link(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get name() {
        const ret = wasm.personsummary_name(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get email() {
        const ret = wasm.personsummary_email(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
    /**
     * @returns {string}
     */
    get orcid() {
        const ret = wasm.personsummary_orcid(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

const PersonUpdateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personupdate_free(ptr >>> 0, 1));

export class PersonUpdate {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonUpdate.prototype);
        obj.__wbg_ptr = ptr;
        PersonUpdateFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            data_update: this.data_update,
            add_roles: this.add_roles,
            remove_roles: this.remove_roles,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonUpdateFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personupdate_free(ptr, 0);
    }
    /**
     * @returns {PersonDataUpdate}
     */
    get data_update() {
        const ret = wasm.__wbg_get_personupdate_data_update(this.__wbg_ptr);
        return PersonDataUpdate.__wrap(ret);
    }
    /**
     * @param {PersonDataUpdate} arg0
     */
    set data_update(arg0) {
        _assertClass(arg0, PersonDataUpdate);
        var ptr0 = arg0.__destroy_into_raw();
        wasm.__wbg_set_personupdate_data_update(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {any[]}
     */
    get add_roles() {
        const ret = wasm.__wbg_get_personupdate_add_roles(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {any[]} arg0
     */
    set add_roles(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_personupdate_add_roles(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {any[]}
     */
    get remove_roles() {
        const ret = wasm.__wbg_get_personupdate_remove_roles(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {any[]} arg0
     */
    set remove_roles(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_personupdate_remove_roles(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {PersonUpdateBuilder}
     */
    static new() {
        const ret = wasm.personupdate_new();
        return PersonUpdateBuilder.__wrap(ret);
    }
}

const PersonUpdateBuilderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personupdatebuilder_free(ptr >>> 0, 1));
/**
 * Builder for [`PersonUpdate`](struct.PersonUpdate.html).
 */
export class PersonUpdateBuilder {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PersonUpdateBuilder.prototype);
        obj.__wbg_ptr = ptr;
        PersonUpdateBuilderFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonUpdateBuilderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personupdatebuilder_free(ptr, 0);
    }
    /**
     * @param {PersonDataUpdate} value
     * @returns {PersonUpdateBuilder}
     */
    data_update(value) {
        const ptr = this.__destroy_into_raw();
        _assertClass(value, PersonDataUpdate);
        var ptr0 = value.__destroy_into_raw();
        const ret = wasm.personupdatebuilder_data_update(ptr, ptr0);
        return PersonUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {any[]} value
     * @returns {PersonUpdateBuilder}
     */
    add_roles(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.personupdatebuilder_add_roles(ptr, ptr0, len0);
        return PersonUpdateBuilder.__wrap(ret);
    }
    /**
     * @param {any[]} value
     * @returns {PersonUpdateBuilder}
     */
    remove_roles(value) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.personupdatebuilder_remove_roles(ptr, ptr0, len0);
        return PersonUpdateBuilder.__wrap(ret);
    }
    /**
     * Builds a new `PersonUpdate`.
     *
     * # Errors
     *
     * If a required field has not been initialized.
     * @returns {PersonUpdate}
     */
    build() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.personupdatebuilder_build(ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return PersonUpdate.__wrap(ret[0]);
    }
}

const PersonUpdateErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_personupdateerror_free(ptr >>> 0, 1));

export class PersonUpdateError {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PersonUpdateErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_personupdateerror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    error() {
        const ret = wasm.personupdateerror_error(this.__wbg_ptr);
        var v1 = getCachedStringFromWasm0(ret[0], ret[1]);
        if (ret[0] !== 0) { wasm.__wbindgen_free(ret[0], ret[1], 1); }
        return v1;
    }
}

export function __wbg_abort_410ec47a64ac6117(arg0, arg1) {
    arg0.abort(arg1);
};

export function __wbg_abort_775ef1d17fc65868(arg0) {
    arg0.abort();
};

export function __wbg_append_8c7dd8d641a5f01b() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    var v1 = getCachedStringFromWasm0(arg3, arg4);
    arg0.append(v0, v1);
}, arguments) };

export function __wbg_arrayBuffer_d1b44c4390db422f() { return handleError(function (arg0) {
    const ret = arg0.arrayBuffer();
    return ret;
}, arguments) };

export function __wbg_buffer_609cc3eee51ed158(arg0) {
    const ret = arg0.buffer;
    return ret;
};

export function __wbg_call_672a4d21634d4a24() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

export function __wbg_call_7cccdd69e0791ae2() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

export function __wbg_clearTimeout_121ece162c044c80(arg0) {
    const ret = clearTimeout(arg0);
    return ret;
};

export function __wbg_createduser_new(arg0) {
    const ret = CreatedUser.__wrap(arg0);
    return ret;
};

export function __wbg_done_769e5ede4b31c67b(arg0) {
    const ret = arg0.done;
    return ret;
};

export function __wbg_fetch_43e69ddf509149f8(arg0) {
    const ret = fetch(arg0);
    return ret;
};

export function __wbg_fetch_509096533071c657(arg0, arg1) {
    const ret = arg0.fetch(arg1);
    return ret;
};

export function __wbg_get_67b2ba62fc30de12() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

export function __wbg_has_a5ea9117f258a0ec() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(arg0, arg1);
    return ret;
}, arguments) };

export function __wbg_headers_9cb51cfd2ac780a4(arg0) {
    const ret = arg0.headers;
    return ret;
};

export function __wbg_instanceof_Response_f2cc20d9f7dfd644(arg0) {
    let result;
    try {
        result = arg0 instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_institution_new(arg0) {
    const ret = Institution.__wrap(arg0);
    return ret;
};

export function __wbg_institutionordering_new(arg0) {
    const ret = InstitutionOrdering.__wrap(arg0);
    return ret;
};

export function __wbg_institutionordering_unwrap(arg0) {
    const ret = InstitutionOrdering.__unwrap(arg0);
    return ret;
};

export function __wbg_iterator_9a24c88df860dc65() {
    const ret = Symbol.iterator;
    return ret;
};

export function __wbg_lab_new(arg0) {
    const ret = Lab.__wrap(arg0);
    return ret;
};

export function __wbg_labordering_new(arg0) {
    const ret = LabOrdering.__wrap(arg0);
    return ret;
};

export function __wbg_labordering_unwrap(arg0) {
    const ret = LabOrdering.__unwrap(arg0);
    return ret;
};

export function __wbg_labupdateerror_new(arg0) {
    const ret = LabUpdateError.__wrap(arg0);
    return ret;
};

export function __wbg_length_a446193dc22c12f8(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_new_018dcc2d6c8c2f6a() { return handleError(function () {
    const ret = new Headers();
    return ret;
}, arguments) };

export function __wbg_new_23a2665fac83c611(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_424(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return ret;
    } finally {
        state0.a = state0.b = 0;
    }
};

export function __wbg_new_405e22f390576ce2() {
    const ret = new Object();
    return ret;
};

export function __wbg_new_5e0be73521bc8c17() {
    const ret = new Map();
    return ret;
};

export function __wbg_new_78feb108b6472713() {
    const ret = new Array();
    return ret;
};

export function __wbg_new_a12002a7f91c75be(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

export function __wbg_new_e25e5aab09ff45db() { return handleError(function () {
    const ret = new AbortController();
    return ret;
}, arguments) };

export function __wbg_newcommitteeapproval_new(arg0) {
    const ret = NewCommitteeApproval.__wrap(arg0);
    return ret;
};

export function __wbg_newcommitteeapproval_unwrap(arg0) {
    const ret = NewCommitteeApproval.__unwrap(arg0);
    return ret;
};

export function __wbg_newcommitteeapprovalerror_new(arg0) {
    const ret = NewCommitteeApprovalError.__wrap(arg0);
    return ret;
};

export function __wbg_newinstitutionerror_new(arg0) {
    const ret = NewInstitutionError.__wrap(arg0);
    return ret;
};

export function __wbg_newlaberror_new(arg0) {
    const ret = NewLabError.__wrap(arg0);
    return ret;
};

export function __wbg_newnoargs_105ed471475aaf50(arg0, arg1) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Function(v0);
    return ret;
};

export function __wbg_newpersonerror_new(arg0) {
    const ret = NewPersonError.__wrap(arg0);
    return ret;
};

export function __wbg_newsamplemetadataerror_new(arg0) {
    const ret = NewSampleMetadataError.__wrap(arg0);
    return ret;
};

export function __wbg_newwithbyteoffsetandlength_d97e637ebe145a9a(arg0, arg1, arg2) {
    const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
    return ret;
};

export function __wbg_newwithstrandinit_06c535e0a867c635() { return handleError(function (arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg0, arg1);
    const ret = new Request(v0, arg2);
    return ret;
}, arguments) };

export function __wbg_next_25feadfc0913fea9(arg0) {
    const ret = arg0.next;
    return ret;
};

export function __wbg_next_6574e1a8a62d1055() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

export function __wbg_person_new(arg0) {
    const ret = Person.__wrap(arg0);
    return ret;
};

export function __wbg_personordering_new(arg0) {
    const ret = PersonOrdering.__wrap(arg0);
    return ret;
};

export function __wbg_personordering_unwrap(arg0) {
    const ret = PersonOrdering.__unwrap(arg0);
    return ret;
};

export function __wbg_personsummary_new(arg0) {
    const ret = PersonSummary.__wrap(arg0);
    return ret;
};

export function __wbg_queueMicrotask_97d92b4fcc8a61c5(arg0) {
    queueMicrotask(arg0);
};

export function __wbg_queueMicrotask_d3219def82552485(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

export function __wbg_resolve_4851785c9c5f573d(arg0) {
    const ret = Promise.resolve(arg0);
    return ret;
};

export function __wbg_setTimeout_e64b2910d9d7169a(arg0, arg1) {
    const ret = setTimeout(arg0, arg1);
    return ret;
};

export function __wbg_set_37837023f3d740e8(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
};

export function __wbg_set_3f1d0b984ed272ed(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

export function __wbg_set_65595bdd868b3009(arg0, arg1, arg2) {
    arg0.set(arg1, arg2 >>> 0);
};

export function __wbg_set_8fc6bf8a5b1071d1(arg0, arg1, arg2) {
    const ret = arg0.set(arg1, arg2);
    return ret;
};

export function __wbg_setbody_5923b78a95eedf29(arg0, arg1) {
    arg0.body = arg1;
};

export function __wbg_setcredentials_c3a22f1cd105a2c6(arg0, arg1) {
    arg0.credentials = __wbindgen_enum_RequestCredentials[arg1];
};

export function __wbg_setheaders_834c0bdb6a8949ad(arg0, arg1) {
    arg0.headers = arg1;
};

export function __wbg_setmethod_3c5280fe5d890842(arg0, arg1, arg2) {
    var v0 = getCachedStringFromWasm0(arg1, arg2);
    arg0.method = v0;
};

export function __wbg_setmode_5dc300b865044b65(arg0, arg1) {
    arg0.mode = __wbindgen_enum_RequestMode[arg1];
};

export function __wbg_setsignal_75b21ef3a81de905(arg0, arg1) {
    arg0.signal = arg1;
};

export function __wbg_signal_aaf9ad74119f20a4(arg0) {
    const ret = arg0.signal;
    return ret;
};

export function __wbg_static_accessor_GLOBAL_88a902d13a557d07() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_SELF_37c5d418e4bf5819() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_WINDOW_5de37043a91a9c40() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_status_f6360336ca686bf0(arg0) {
    const ret = arg0.status;
    return ret;
};

export function __wbg_stringify_f7ed6987935b4a24() { return handleError(function (arg0) {
    const ret = JSON.stringify(arg0);
    return ret;
}, arguments) };

export function __wbg_then_44b73946d2fb3e7d(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

export function __wbg_then_48b406749878a531(arg0, arg1, arg2) {
    const ret = arg0.then(arg1, arg2);
    return ret;
};

export function __wbg_url_ae10c34ca209681d(arg0, arg1) {
    const ret = arg1.url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_value_cd1ffa7b1ab794f1(arg0) {
    const ret = arg0.value;
    return ret;
};

export function __wbindgen_bigint_from_i64(arg0) {
    const ret = arg0;
    return ret;
};

export function __wbindgen_bigint_from_u64(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return ret;
};

export function __wbindgen_cb_drop(arg0) {
    const obj = arg0.original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

export function __wbindgen_closure_wrapper879(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 104, __wbg_adapter_40);
    return ret;
};

export function __wbindgen_closure_wrapper935(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 124, __wbg_adapter_43);
    return ret;
};

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbindgen_error_new(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_export_0;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

export function __wbindgen_is_function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

export function __wbindgen_is_object(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbindgen_is_string(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

export function __wbindgen_is_undefined(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

export function __wbindgen_memory() {
    const ret = wasm.memory;
    return ret;
};

export function __wbindgen_number_get(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return ret;
};

export function __wbindgen_string_get(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbindgen_try_into_number(arg0) {
    let result;
    try { result = +arg0 } catch (e) { result = e }
    const ret = result;
    return ret;
};

