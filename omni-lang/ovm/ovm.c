/* OVM Runtime — Standalone Omni Virtual Machine
 * 
 * Loads and executes OVM bytecode (.ovm files) without any Rust dependency.
 * This is the foundation that makes Omni a standalone, self-hosting language.
 * 
 * Build:  cc -O2 -o ovm ovm.c
 * Usage:  ./ovm program.ovm [args...]
 * 
 * Binary format (little-endian):
 *   [4] magic "OVM\0"
 *   [4] version (u32)
 *   [4] flags (u32)
 *   [8] entry_point (u64) — function index
 *   [8] const_pool_off, [8] const_pool_len
 *   [8] code_off, [8] code_len
 *   [8] symbol_off, [8] symbol_len
 *   [8] debug_off, [8] debug_len
 *   [const_pool_len bytes] constant pool
 *   [code_len bytes] code section
 *   [symbol_len bytes] symbol table
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <math.h>

/* ─── Configuration ─────────────────────────────────────────────── */
#define STACK_MAX     65536
#define CALL_MAX      1024
#define HEAP_INIT     4096
#define NATIVE_MAX    64

/* ─── Opcodes ───────────────────────────────────────────────────── */
enum {
    OP_PUSH_I8    = 0x01, OP_PUSH_I16,  OP_PUSH_I32,  OP_PUSH_I64,
    OP_PUSH_F32   = 0x05, OP_PUSH_F64,
    OP_PUSH_STR   = 0x07,
    OP_PUSH_NULL  = 0x08, OP_PUSH_TRUE, OP_PUSH_FALSE,
    OP_DUP        = 0x0B, OP_DUP2, OP_SWAP, OP_ROT,
    OP_LOAD_CONST = 0x0E,
    OP_POP        = 0x0F,
    OP_ADD_I64    = 0x10, OP_SUB_I64,   OP_MUL_I64,   OP_DIV_I64,  OP_MOD_I64,
    OP_ADD_F64    = 0x18, OP_SUB_F64,   OP_MUL_F64,   OP_DIV_F64,  OP_MOD_F64,
    OP_NEG_I64    = 0x20, OP_NEG_F64,
    OP_INC        = 0x22, OP_DEC,
    OP_EQ         = 0x30, OP_NE, OP_LT, OP_LE, OP_GT, OP_GE,
    OP_CMP        = 0x36, OP_IS_NULL,
    OP_AND        = 0x38, OP_OR, OP_XOR, OP_NOT, OP_SHL, OP_SHR,
    OP_JMP        = 0x50, OP_JMP_ABS,
    OP_JZ         = 0x52, OP_JNZ, OP_JLT, OP_JLE, OP_JGT, OP_JGE,
    OP_CALL       = 0x58, OP_CALL_IND, OP_RET, OP_RET_VOID, OP_TAIL_CALL,
    OP_LOAD_LOC   = 0x60, OP_STORE_LOC,
    OP_LOAD_ARG   = 0x62, OP_STORE_ARG,
    OP_ALLOC_LOC  = 0x64, OP_FREE_LOC,
    OP_LOAD_GLB   = 0x66, OP_STORE_GLB,
    OP_LOAD_REG   = 0x70, OP_STORE_REG, OP_MOVE_REG, OP_SWAP_REG,
    OP_ALLOC      = 0x80, OP_FREE, OP_REALLOC,
    OP_LOAD_8     = 0x84, OP_LOAD_16, OP_LOAD_32, OP_LOAD_64,
    OP_STORE_8    = 0x88, OP_STORE_16, OP_STORE_32, OP_STORE_64,
    OP_MEMCPY     = 0x8C, OP_MEMSET,
    OP_NEW        = 0x90, OP_GET_FIELD, OP_SET_FIELD, OP_INSTANCEOF, OP_CAST,
    OP_NEW_ARRAY  = 0x95, OP_ARRAY_LEN, OP_ARRAY_GET, OP_ARRAY_SET, OP_ARRAY_SLICE,
    OP_I2F        = 0xA0, OP_F2I, OP_I2B, OP_B2I,
    OP_LOAD_CONST_B = 0xB0,
    OP_SYSCALL    = 0xF0,
    OP_HALT       = 0xFE,
    OP_PANIC      = 0xFD,
    OP_DEBUG      = 0xFB, OP_TRACE, OP_ASSERT,
};

/* ─── Value types ───────────────────────────────────────────────── */
typedef enum {
    VAL_NULL,
    VAL_I64,
    VAL_F64,
    VAL_BOOL,
    VAL_STRING,
    VAL_ARRAY,
    VAL_PTR,
} ValTag;

typedef struct Value {
    ValTag tag;
    union {
        int64_t  i64;
        double   f64;
        int      bool_val;
        char    *str;
        struct {
            struct Value *items;
            int len;
            int cap;
        } arr;
        void *ptr;
    } as;
} Value;

/* ─── Native function signature ─────────────────────────────────── */
typedef Value (*NativeFn)(Value *args, int nargs);

/* ─── VM state ──────────────────────────────────────────────────── */
typedef struct {
    /* Bytecode */
    uint8_t *code;
    uint64_t code_len;
    
    /* Constant pool */
    Value   *consts;
    uint32_t const_count;
    
    /* Symbol table */
    char   **func_names;
    uint64_t *func_addrs;
    uint32_t  func_count;
    uint64_t  entry_point;
    
    /* Execution */
    Value    stack[STACK_MAX];
    int      sp;
    
    /* Call frames */
    struct {
        uint64_t return_pc;
        int      stack_base;
        int      num_locals;
        Value    locals[256];
    } frames[CALL_MAX];
    int fp;
    
    uint64_t pc;
    int      running;
    
    /* Native functions */
    struct {
        char     *name;
        NativeFn  fn;
    } natives[NATIVE_MAX];
    int native_count;
    
    /* String pool (prevent leaks) */
    char    *strings[4096];
    int      string_count;
    
    /* Program args */
    int      argc;
    char   **argv;
} OVM;

/* ─── String helpers ────────────────────────────────────────────── */
static char *ovm_strdup(OVM *vm, const char *s) {
    char *d = strdup(s);
    if (vm->string_count < 4096)
        vm->strings[vm->string_count++] = d;
    return d;
}

/* ─── Value constructors ────────────────────────────────────────── */
static Value v_null(void)  { return (Value){.tag = VAL_NULL}; }
static Value v_i64(int64_t v) { return (Value){.tag = VAL_I64, .as.i64 = v}; }
static Value v_f64(double v)  { return (Value){.tag = VAL_F64, .as.f64 = v}; }
static Value v_bool(int v)    { return (Value){.tag = VAL_BOOL, .as.bool_val = v}; }
static Value v_str(OVM *vm, const char *s) {
    return (Value){.tag = VAL_STRING, .as.str = ovm_strdup(vm, s)};
}

static int64_t val_as_int(Value v) {
    if (v.tag == VAL_I64) return v.as.i64;
    if (v.tag == VAL_F64) return (int64_t)v.as.f64;
    if (v.tag == VAL_BOOL) return v.as.bool_val;
    return 0;
}

static double val_as_float(Value v) {
    if (v.tag == VAL_F64) return v.as.f64;
    if (v.tag == VAL_I64) return (double)v.as.i64;
    return 0.0;
}

static int val_truthy(Value v) {
    switch (v.tag) {
        case VAL_NULL:  return 0;
        case VAL_BOOL:  return v.as.bool_val;
        case VAL_I64:   return v.as.i64 != 0;
        case VAL_F64:   return v.as.f64 != 0.0;
        case VAL_STRING: return v.as.str && v.as.str[0] != '\0';
        default:        return 1;
    }
}

/* ─── Stack operations ──────────────────────────────────────────── */
static void push(OVM *vm, Value v) {
    if (vm->sp >= STACK_MAX) { fprintf(stderr, "OVM: Stack overflow\n"); exit(1); }
    vm->stack[vm->sp++] = v;
}

static Value pop(OVM *vm) {
    if (vm->sp <= 0) { fprintf(stderr, "OVM: Stack underflow\n"); exit(1); }
    return vm->stack[--vm->sp];
}

static Value peek(OVM *vm, int offset) {
    return vm->stack[vm->sp - 1 - offset];
}

/* ─── Byte reading helpers ──────────────────────────────────────── */
static uint8_t  read_u8(OVM *vm)  { return vm->code[vm->pc++]; }
static uint16_t read_u16(OVM *vm) {
    uint16_t v = vm->code[vm->pc] | (vm->code[vm->pc+1] << 8);
    vm->pc += 2;
    return v;
}
static uint32_t read_u32(OVM *vm) {
    uint32_t v = 0;
    memcpy(&v, &vm->code[vm->pc], 4);
    vm->pc += 4;
    return v;
}
static uint64_t read_u64(OVM *vm) {
    uint64_t v = 0;
    memcpy(&v, &vm->code[vm->pc], 8);
    vm->pc += 8;
    return v;
}
static int64_t  read_i64(OVM *vm) { return (int64_t)read_u64(vm); }
static double   read_f64(OVM *vm) {
    double v;
    memcpy(&v, &vm->code[vm->pc], 8);
    vm->pc += 8;
    return v;
}

/* ─── Native function registration ──────────────────────────────── */
static void register_native(OVM *vm, const char *name, NativeFn fn) {
    if (vm->native_count >= NATIVE_MAX) return;
    vm->natives[vm->native_count].name = strdup(name);
    vm->natives[vm->native_count].fn = fn;
    vm->native_count++;
}

static NativeFn find_native(OVM *vm, const char *name) {
    for (int i = 0; i < vm->native_count; i++) {
        if (strcmp(vm->natives[i].name, name) == 0)
            return vm->natives[i].fn;
    }
    return NULL;
}

/* ─── Built-in native functions ─────────────────────────────────── */

static Value native_print(Value *args, int nargs) {
    for (int i = 0; i < nargs; i++) {
        switch (args[i].tag) {
            case VAL_I64:    printf("%lld", (long long)args[i].as.i64); break;
            case VAL_F64:    printf("%g", args[i].as.f64); break;
            case VAL_BOOL:   printf("%s", args[i].as.bool_val ? "true" : "false"); break;
            case VAL_STRING: printf("%s", args[i].as.str ? args[i].as.str : ""); break;
            case VAL_NULL:   printf("null"); break;
            default:         printf("<??>"); break;
        }
    }
    return v_null();
}

static Value native_println(Value *args, int nargs) {
    native_print(args, nargs);
    printf("\n");
    fflush(stdout);
    return v_null();
}

static Value native_to_string(Value *args, int nargs) {
    if (nargs < 1) return v_str(NULL, "");
    char buf[64];
    switch (args[0].tag) {
        case VAL_I64:    snprintf(buf, sizeof(buf), "%lld", (long long)args[0].as.i64); break;
        case VAL_F64:    snprintf(buf, sizeof(buf), "%g", args[0].as.f64); break;
        case VAL_BOOL:   snprintf(buf, sizeof(buf), "%s", args[0].as.bool_val ? "true" : "false"); break;
        case VAL_STRING: return args[0];
        case VAL_NULL:   return v_str(NULL, "null");
        default:         return v_str(NULL, "<??>");
    }
    return v_str(NULL, buf);
}

static Value native_to_int(Value *args, int nargs) {
    if (nargs < 1) return v_i64(0);
    return v_i64(val_as_int(args[0]));
}

static Value native_to_float(Value *args, int nargs) {
    if (nargs < 1) return v_f64(0.0);
    return v_f64(val_as_float(args[0]));
}

static Value native_typeof(Value *args, int nargs) {
    if (nargs < 1) return v_str(NULL, "null");
    switch (args[0].tag) {
        case VAL_I64:    return v_str(NULL, "int");
        case VAL_F64:    return v_str(NULL, "float");
        case VAL_BOOL:   return v_str(NULL, "bool");
        case VAL_STRING: return v_str(NULL, "string");
        case VAL_ARRAY:  return v_str(NULL, "array");
        case VAL_NULL:   return v_str(NULL, "null");
        default:         return v_str(NULL, "unknown");
    }
}

static Value native_assert(Value *args, int nargs) {
    if (nargs < 1 || !val_truthy(args[0])) {
        fprintf(stderr, "Assertion failed\n");
        exit(1);
    }
    return v_null();
}

static Value native_len(Value *args, int nargs) {
    if (nargs < 1) return v_i64(0);
    if (args[0].tag == VAL_STRING) return v_i64((int64_t)strlen(args[0].as.str));
    if (args[0].tag == VAL_ARRAY)  return v_i64(args[0].as.arr.len);
    return v_i64(0);
}

static Value native_format(Value *args, int nargs) {
    /* Simple format: convert first arg to string */
    if (nargs < 1) return v_str(NULL, "");
    char buf[256];
    switch (args[0].tag) {
        case VAL_I64:    snprintf(buf, sizeof(buf), "%lld", (long long)args[0].as.i64); break;
        case VAL_F64:    snprintf(buf, sizeof(buf), "%g", args[0].as.f64); break;
        case VAL_BOOL:   snprintf(buf, sizeof(buf), "%s", args[0].as.bool_val ? "true" : "false"); break;
        case VAL_STRING: return args[0];
        default:         return v_str(NULL, "");
    }
    return v_str(NULL, buf);
}

static Value native_int_div(Value *args, int nargs) {
    if (nargs < 2) return v_i64(0);
    int64_t b = val_as_int(args[1]);
    if (b == 0) { fprintf(stderr, "Division by zero\n"); exit(1); }
    return v_i64(val_as_int(args[0]) / b);
}

static Value native_int_mod(Value *args, int nargs) {
    if (nargs < 2) return v_i64(0);
    int64_t b = val_as_int(args[1]);
    if (b == 0) { fprintf(stderr, "Modulo by zero\n"); exit(1); }
    return v_i64(val_as_int(args[0]) % b);
}

static Value native_sqrt(Value *args, int nargs) {
    if (nargs < 1) return v_f64(0.0);
    return v_f64(sqrt(val_as_float(args[0])));
}

static Value native_abs(Value *args, int nargs) {
    if (nargs < 1) return v_i64(0);
    if (args[0].tag == VAL_F64) return v_f64(fabs(args[0].as.f64));
    int64_t v = val_as_int(args[0]);
    return v_i64(v < 0 ? -v : v);
}

static Value native_pow(Value *args, int nargs) {
    if (nargs < 2) return v_f64(0.0);
    return v_f64(pow(val_as_float(args[0]), val_as_float(args[1])));
}

static Value native_min(Value *args, int nargs) {
    if (nargs < 2) return args[0];
    int64_t a = val_as_int(args[0]), b = val_as_int(args[1]);
    return v_i64(a < b ? a : b);
}

static Value native_max(Value *args, int nargs) {
    if (nargs < 2) return args[0];
    int64_t a = val_as_int(args[0]), b = val_as_int(args[1]);
    return v_i64(a > b ? a : b);
}

static Value native_exit(Value *args, int nargs) {
    int code = (nargs > 0) ? (int)val_as_int(args[0]) : 0;
    exit(code);
    return v_null(); /* unreachable */
}

static Value native_string_concat(Value *args, int nargs) {
    if (nargs < 2) return args[0];
    const char *a = (args[0].tag == VAL_STRING) ? args[0].as.str : "";
    const char *b = (args[1].tag == VAL_STRING) ? args[1].as.str : "";
    size_t la = strlen(a), lb = strlen(b);
    char *r = malloc(la + lb + 1);
    memcpy(r, a, la);
    memcpy(r + la, b, lb);
    r[la + lb] = '\0';
    /* Note: leaks in this simplified version — production would use arena */
    return (Value){.tag = VAL_STRING, .as.str = r};
}

/* ─── Bytecode loader ───────────────────────────────────────────── */
static int load_ovm(OVM *vm, const uint8_t *data, size_t size) {
    if (size < 64) {
        fprintf(stderr, "OVM: File too small\n");
        return -1;
    }
    
    /* Check magic */
    if (memcmp(data, "OVM\0", 4) != 0) {
        fprintf(stderr, "OVM: Invalid magic number\n");
        return -1;
    }
    
    uint32_t version;
    memcpy(&version, data + 4, 4);
    if (version != 1) {
        fprintf(stderr, "OVM: Unsupported version %u\n", version);
        return -1;
    }
    
    /* Parse header */
    uint32_t flags;
    memcpy(&flags, data + 8, 4);
    
    memcpy(&vm->entry_point, data + 12, 8);
    
    uint64_t const_off, const_len, code_off, code_len;
    memcpy(&const_off,  data + 24, 8);
    memcpy(&const_len,  data + 32, 8);
    memcpy(&code_off,   data + 40, 8);
    memcpy(&code_len,   data + 48, 8);
    
    /* Load constant pool */
    if (const_off > 0 && const_len > 0) {
        const uint8_t *cp = data + const_off;
        uint32_t count;
        memcpy(&count, cp, 4);
        cp += 4;
        
        vm->const_count = count;
        vm->consts = calloc(count, sizeof(Value));
        
        for (uint32_t i = 0; i < count; i++) {
            uint8_t tag = *cp++;
            switch (tag) {
                case 0x01: { /* I64 */
                    int64_t v;
                    memcpy(&v, cp, 8);
                    cp += 8;
                    vm->consts[i] = v_i64(v);
                    break;
                }
                case 0x02: { /* F64 */
                    double v;
                    memcpy(&v, cp, 8);
                    cp += 8;
                    vm->consts[i] = v_f64(v);
                    break;
                }
                case 0x03: { /* String */
                    uint32_t len;
                    memcpy(&len, cp, 4);
                    cp += 4;
                    char *s = malloc(len + 1);
                    memcpy(s, cp, len);
                    s[len] = '\0';
                    cp += len;
                    vm->consts[i] = (Value){.tag = VAL_STRING, .as.str = s};
                    break;
                }
                case 0x04: { /* Bytes */
                    uint32_t len;
                    memcpy(&len, cp, 4);
                    cp += 4;
                    cp += len; /* skip */
                    vm->consts[i] = v_null();
                    break;
                }
                default:
                    vm->consts[i] = v_null();
                    break;
            }
        }
    }
    
    /* Load code */
    if (code_off > 0 && code_len > 0) {
        vm->code = malloc(code_len);
        memcpy(vm->code, data + code_off, code_len);
        vm->code_len = code_len;
    }
    
    /* Load symbol table (function names -> code offsets) */
    uint64_t sym_off, sym_len;
    memcpy(&sym_off, data + 56, 8);
    memcpy(&sym_len, data + 64, 8);
    
    if (sym_off > 0 && sym_len > 0) {
        const uint8_t *sp = data + sym_off;
        uint32_t count;
        memcpy(&count, sp, 4);
        sp += 4;
        
        vm->func_count = count;
        vm->func_names = calloc(count, sizeof(char*));
        vm->func_addrs = calloc(count, sizeof(uint64_t));
        
        for (uint32_t i = 0; i < count; i++) {
            /* name_len (u32) + name + code_offset (u64) */
            uint32_t nlen;
            memcpy(&nlen, sp, 4);
            sp += 4;
            char *name = malloc(nlen + 1);
            memcpy(name, sp, nlen);
            name[nlen] = '\0';
            sp += nlen;
            uint64_t addr;
            memcpy(&addr, sp, 8);
            sp += 8;
            
            vm->func_names[i] = name;
            vm->func_addrs[i] = addr;
        }
    }
    
    return 0;
}

/* ─── Find function index by name ───────────────────────────────── */
static int find_func(OVM *vm, const char *name) {
    for (uint32_t i = 0; i < vm->func_count; i++) {
        if (strcmp(vm->func_names[i], name) == 0)
            return (int)i;
    }
    return -1;
}

/* ─── Main execution loop ───────────────────────────────────────── */
static Value execute(OVM *vm) {
    while (vm->running && vm->pc < vm->code_len) {
        uint8_t op = vm->code[vm->pc++];
        
        switch (op) {
            /* ── Push constants ── */
            case OP_PUSH_I64: push(vm, v_i64(read_i64(vm))); break;
            case OP_PUSH_F64: push(vm, v_f64(read_f64(vm))); break;
            case OP_PUSH_I8:  push(vm, v_i64((int8_t)read_u8(vm))); break;
            case OP_PUSH_I16: push(vm, v_i64((int16_t)read_u16(vm))); break;
            case OP_PUSH_I32: push(vm, v_i64((int32_t)read_u32(vm))); break;
            case OP_PUSH_NULL:  push(vm, v_null()); break;
            case OP_PUSH_TRUE:  push(vm, v_bool(1)); break;
            case OP_PUSH_FALSE: push(vm, v_bool(0)); break;
            case OP_LOAD_CONST: {
                uint32_t idx = read_u32(vm);
                if (idx < vm->const_count)
                    push(vm, vm->consts[idx]);
                else
                    push(vm, v_null());
                break;
            }
            case OP_LOAD_CONST_B: {
                uint8_t idx = read_u8(vm);
                if (idx < vm->const_count)
                    push(vm, vm->consts[idx]);
                else
                    push(vm, v_null());
                break;
            }
            case OP_POP: pop(vm); break;
            case OP_DUP: { Value v = peek(vm, 0); push(vm, v); break; }
            case OP_SWAP: {
                Value a = pop(vm), b = pop(vm);
                push(vm, a); push(vm, b);
                break;
            }
            
            /* ── Arithmetic (i64) ── */
            case OP_ADD_I64: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_i64(l + r)); break; }
            case OP_SUB_I64: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_i64(l - r)); break; }
            case OP_MUL_I64: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_i64(l * r)); break; }
            case OP_DIV_I64: {
                int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm));
                if (r == 0) { fprintf(stderr, "Division by zero\n"); exit(1); }
                push(vm, v_i64(l / r));
                break;
            }
            case OP_MOD_I64: {
                int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm));
                if (r == 0) { fprintf(stderr, "Modulo by zero\n"); exit(1); }
                push(vm, v_i64(l % r));
                break;
            }
            
            /* ── Arithmetic (f64) ── */
            case OP_ADD_F64: { double r = val_as_float(pop(vm)), l = val_as_float(pop(vm)); push(vm, v_f64(l + r)); break; }
            case OP_SUB_F64: { double r = val_as_float(pop(vm)), l = val_as_float(pop(vm)); push(vm, v_f64(l - r)); break; }
            case OP_MUL_F64: { double r = val_as_float(pop(vm)), l = val_as_float(pop(vm)); push(vm, v_f64(l * r)); break; }
            case OP_DIV_F64: { double r = val_as_float(pop(vm)), l = val_as_float(pop(vm)); push(vm, v_f64(l / r)); break; }
            
            /* ── Negation ── */
            case OP_NEG_I64: { Value v = pop(vm); push(vm, v_i64(-val_as_int(v))); break; }
            case OP_NEG_F64: { Value v = pop(vm); push(vm, v_f64(-val_as_float(v))); break; }
            case OP_INC: { Value v = pop(vm); push(vm, v_i64(val_as_int(v) + 1)); break; }
            case OP_DEC: { Value v = pop(vm); push(vm, v_i64(val_as_int(v) - 1)); break; }
            
            /* ── Comparison ── */
            case OP_EQ: { Value r = pop(vm), l = pop(vm);
                if (l.tag == VAL_I64 && r.tag == VAL_I64) push(vm, v_bool(l.as.i64 == r.as.i64));
                else if (l.tag == VAL_STRING && r.tag == VAL_STRING) push(vm, v_bool(strcmp(l.as.str, r.as.str) == 0));
                else push(vm, v_bool(val_as_int(l) == val_as_int(r)));
                break;
            }
            case OP_NE: { Value r = pop(vm), l = pop(vm);
                push(vm, v_bool(val_truthy(l) != val_truthy(r)));
                break;
            }
            case OP_LT: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_bool(l < r)); break; }
            case OP_LE: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_bool(l <= r)); break; }
            case OP_GT: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_bool(l > r)); break; }
            case OP_GE: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_bool(l >= r)); break; }
            
            /* ── Logical ── */
            case OP_AND: { int r = val_truthy(pop(vm)), l = val_truthy(pop(vm)); push(vm, v_bool(l && r)); break; }
            case OP_OR:  { int r = val_truthy(pop(vm)), l = val_truthy(pop(vm)); push(vm, v_bool(l || r)); break; }
            case OP_NOT: { push(vm, v_bool(!val_truthy(pop(vm)))); break; }
            case OP_IS_NULL: { push(vm, v_bool(pop(vm).tag == VAL_NULL)); break; }
            
            /* ── Bitwise ── */
            case OP_XOR: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_i64(l ^ r)); break; }
            case OP_SHL: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_i64(l << r)); break; }
            case OP_SHR: { int64_t r = val_as_int(pop(vm)), l = val_as_int(pop(vm)); push(vm, v_i64(l >> r)); break; }
            
            /* ── Jumps ── */
            case OP_JMP: {
                int32_t offset = (int32_t)read_u32(vm);
                vm->pc = (uint64_t)((int64_t)vm->pc + offset);
                break;
            }
            case OP_JMP_ABS: { vm->pc = read_u64(vm); break; }
            case OP_JZ:  { int32_t off = (int32_t)read_u32(vm); if (!val_truthy(pop(vm))) vm->pc = (uint64_t)((int64_t)vm->pc + off); break; }
            case OP_JNZ: { int32_t off = (int32_t)read_u32(vm); if (val_truthy(pop(vm)))  vm->pc = (uint64_t)((int64_t)vm->pc + off); break; }
            
            /* ── Calls ── */
            case OP_CALL: {
                uint32_t func_idx = read_u32(vm);
                if (func_idx >= vm->func_count) {
                    fprintf(stderr, "OVM: Invalid function index %u\n", func_idx);
                    exit(1);
                }
                /* Check if it's a native function first */
                const char *fname = vm->func_names[func_idx];
                NativeFn native = find_native(vm, fname);
                if (native) {
                    /* Gather args from stack (they were pushed before CALL) */
                    /* We don't know nargs here — use 0 for now (natives check) */
                    Value result = native(NULL, 0);
                    push(vm, result);
                    break;
                }
                
                /* Push call frame */
                if (vm->fp >= CALL_MAX) {
                    fprintf(stderr, "OVM: Call stack overflow\n");
                    exit(1);
                }
                vm->frames[vm->fp].return_pc = vm->pc;
                vm->frames[vm->fp].stack_base = vm->sp;
                vm->fp++;
                vm->pc = vm->func_addrs[func_idx];
                break;
            }
            case OP_CALL_IND: {
                /* Indirect call — function index from stack */
                Value v = pop(vm);
                uint32_t func_idx = (uint32_t)val_as_int(v);
                if (func_idx >= vm->func_count) {
                    push(vm, v_null());
                    break;
                }
                NativeFn native = find_native(vm, vm->func_names[func_idx]);
                if (native) {
                    push(vm, native(NULL, 0));
                    break;
                }
                if (vm->fp >= CALL_MAX) { fprintf(stderr, "OVM: Call stack overflow\n"); exit(1); }
                vm->frames[vm->fp].return_pc = vm->pc;
                vm->frames[vm->fp].stack_base = vm->sp;
                vm->fp++;
                vm->pc = vm->func_addrs[func_idx];
                break;
            }
            case OP_RET: {
                Value retval = pop(vm);
                if (vm->fp <= 0) { vm->running = 0; return retval; }
                vm->fp--;
                vm->sp = vm->frames[vm->fp].stack_base;
                vm->pc = vm->frames[vm->fp].return_pc;
                push(vm, retval);
                break;
            }
            case OP_RET_VOID: {
                if (vm->fp <= 0) { vm->running = 0; return v_null(); }
                vm->fp--;
                vm->sp = vm->frames[vm->fp].stack_base;
                vm->pc = vm->frames[vm->fp].return_pc;
                push(vm, v_null());
                break;
            }
            
            /* ── Locals ── */
            case OP_LOAD_LOC: {
                uint8_t idx = read_u8(vm);
                if (vm->fp > 0 && idx < 256)
                    push(vm, vm->frames[vm->fp - 1].locals[idx]);
                else
                    push(vm, v_null());
                break;
            }
            case OP_STORE_LOC: {
                uint8_t idx = read_u8(vm);
                if (vm->fp > 0 && idx < 256)
                    vm->frames[vm->fp - 1].locals[idx] = pop(vm);
                else
                    pop(vm);
                break;
            }
            
            /* ── System calls ── */
            case OP_SYSCALL: {
                uint8_t id = read_u8(vm);
                switch (id) {
                    case 0: /* exit */
                        vm->running = 0;
                        return v_null();
                    case 1: /* print */
                        native_print(vm->stack + vm->sp - 1, 1);
                        break;
                    default:
                        break;
                }
                break;
            }
            
            /* ── Array operations ── */
            case OP_NEW_ARRAY: {
                uint32_t count = read_u32(vm);
                Value arr = {.tag = VAL_ARRAY};
                arr.as.arr.cap = count > 0 ? count : 8;
                arr.as.arr.len = 0;
                arr.as.arr.items = calloc(arr.as.arr.cap, sizeof(Value));
                push(vm, arr);
                break;
            }
            case OP_ARRAY_LEN: {
                Value v = pop(vm);
                if (v.tag == VAL_ARRAY) push(vm, v_i64(v.as.arr.len));
                else push(vm, v_i64(0));
                break;
            }
            
            /* ── Conversion ── */
            case OP_I2F: { push(vm, v_f64((double)val_as_int(pop(vm)))); break; }
            case OP_F2I: { push(vm, v_i64((int64_t)val_as_float(pop(vm)))); break; }
            case OP_I2B: { push(vm, v_bool(val_truthy(pop(vm)))); break; }
            case OP_B2I: { Value v = pop(vm); push(vm, v_i64(v.as.bool_val ? 1 : 0)); break; }
            
            /* ── Control ── */
            case OP_HALT:  vm->running = 0; return v_null();
            case OP_PANIC: {
                Value msg = pop(vm);
                fprintf(stderr, "Panic: %s\n", msg.tag == VAL_STRING ? msg.as.str : "unknown");
                exit(1);
            }
            
            default:
                fprintf(stderr, "OVM: Unknown opcode 0x%02X at pc %llu\n", op, (unsigned long long)vm->pc - 1);
                exit(1);
        }
    }
    
    return v_null();
}

/* ─── Main entry ────────────────────────────────────────────────── */
int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "OVM — Omni Virtual Machine v1.0.0\n");
        fprintf(stderr, "Usage: ovm <program.ovm> [args...]\n");
        fprintf(stderr, "       ovm --version\n");
        return 1;
    }
    
    if (strcmp(argv[1], "--version") == 0 || strcmp(argv[1], "-v") == 0) {
        printf("ovm 1.0.0 (Omni Virtual Machine — Standalone)\n");
        return 0;
    }
    
    /* Read .ovm file */
    FILE *f = fopen(argv[1], "rb");
    if (!f) {
        fprintf(stderr, "OVM: Cannot open '%s'\n", argv[1]);
        return 1;
    }
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    uint8_t *data = malloc(fsize);
    fread(data, 1, fsize, f);
    fclose(f);
    
    /* Initialize VM */
    OVM vm;
    memset(&vm, 0, sizeof(vm));
    vm.running = 1;
    vm.argc = argc - 2;
    vm.argv = argv + 2;
    
    /* Register native functions */
    register_native(&vm, "print",   native_print);
    register_native(&vm, "println", native_println);
    register_native(&vm, "to_string", native_to_string);
    register_native(&vm, "str",     native_to_string);
    register_native(&vm, "int",     native_to_int);
    register_native(&vm, "float",   native_to_float);
    register_native(&vm, "type_of", native_typeof);
    register_native(&vm, "assert",  native_assert);
    register_native(&vm, "len",     native_len);
    register_native(&vm, "format",  native_format);
    register_native(&vm, "sqrt",    native_sqrt);
    register_native(&vm, "abs",     native_abs);
    register_native(&vm, "pow",     native_pow);
    register_native(&vm, "min",     native_min);
    register_native(&vm, "max",     native_max);
    register_native(&vm, "exit",    native_exit);
    
    /* Load bytecode */
    if (load_ovm(&vm, data, fsize) != 0) {
        fprintf(stderr, "OVM: Failed to load '%s'\n", argv[1]);
        free(data);
        return 1;
    }
    
    free(data);
    
    /* Find and call main */
    int main_idx = find_func(&vm, "main");
    if (main_idx < 0) {
        /* Use entry_point from header */
        main_idx = (int)vm.entry_point;
    }
    
    if (main_idx >= 0 && main_idx < (int)vm.func_count) {
        vm.pc = vm.func_addrs[main_idx];
        Value result = execute(&vm);
        (void)result;
    } else {
        fprintf(stderr, "OVM: No entry point found\n");
        return 1;
    }
    
    /* Cleanup */
    for (int i = 0; i < vm.string_count; i++)
        free(vm.strings[i]);
    free(vm.code);
    free(vm.consts);
    for (uint32_t i = 0; i < vm.func_count; i++)
        free(vm.func_names[i]);
    free(vm.func_names);
    free(vm.func_addrs);
    
    return 0;
}
