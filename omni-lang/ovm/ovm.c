/* OVM Runtime — Standalone Omni Virtual Machine
 * 
 * Loads and executes OVM bytecode (.ovm files) without any Rust dependency.
 * This is the foundation that makes Omni a standalone, self-hosting language.
 * 
 * Build:  cc -O2 -std=c11 -o ovm ovm.c -lpthread -lm
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
 *
 * Opcodes match the Rust OVM runner (src/main.rs) exactly.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdatomic.h>
#include <math.h>

/* ─── Platform threading abstraction ────────────────────────────── */
#ifdef _WIN32
  #define WIN32_LEAN_AND_MEAN
  #include <windows.h>

  typedef HANDLE              ovm_thread_t;
  typedef CRITICAL_SECTION    ovm_mutex_t;
  typedef CONDITION_VARIABLE  ovm_cond_t;
  typedef DWORD (WINAPI *ovm_thread_fn)(LPVOID);

  static int ovm_thread_create(ovm_thread_t *t, DWORD (WINAPI *fn)(LPVOID), void *arg) {
      *t = CreateThread(NULL, 0, fn, arg, 0, NULL);
      return (*t == NULL) ? -1 : 0;
  }
  static int ovm_thread_join(ovm_thread_t t) {
      WaitForSingleObject(t, INFINITE);
      CloseHandle(t);
      return 0;
  }
  static void ovm_mutex_init(ovm_mutex_t *m)    { InitializeCriticalSection(m); }
  static void ovm_mutex_destroy(ovm_mutex_t *m)  { DeleteCriticalSection(m); }
  static void ovm_mutex_lock(ovm_mutex_t *m)     { EnterCriticalSection(m); }
  static void ovm_mutex_unlock(ovm_mutex_t *m)   { LeaveCriticalSection(m); }
  static int  ovm_mutex_trylock(ovm_mutex_t *m)  { return TryEnterCriticalSection(m) ? 0 : -1; }
  static void ovm_cond_init(ovm_cond_t *c)       { InitializeConditionVariable(c); }
  static void ovm_cond_destroy(ovm_cond_t *c)    { (void)c; /* no-op on Windows */ }
  static void ovm_cond_wait(ovm_cond_t *c, ovm_mutex_t *m) {
      SleepConditionVariableCS(c, m, INFINITE);
  }
  static void ovm_cond_signal(ovm_cond_t *c)     { WakeConditionVariable(c); }
  static void ovm_cond_broadcast(ovm_cond_t *c)  { WakeAllConditionVariable(c); }

  static void ovm_sleep_ms(int64_t ms) { Sleep((DWORD)ms); }
  static void ovm_thread_yield_cpu(void) { SwitchToThread(); }

#else
  #include <pthread.h>
  #include <unistd.h>
  #include <time.h>
  #include <errno.h>
  #include <sched.h>

  typedef pthread_t         ovm_thread_t;
  typedef pthread_mutex_t   ovm_mutex_t;
  typedef pthread_cond_t    ovm_cond_t;

  static int ovm_thread_create(ovm_thread_t *t, void *(*fn)(void*), void *arg) {
      return pthread_create(t, NULL, fn, arg);
  }
  static int ovm_thread_join(ovm_thread_t t) {
      return pthread_join(t, NULL);
  }
  static void ovm_mutex_init(ovm_mutex_t *m)    { pthread_mutex_init(m, NULL); }
  static void ovm_mutex_destroy(ovm_mutex_t *m)  { pthread_mutex_destroy(m); }
  static void ovm_mutex_lock(ovm_mutex_t *m)     { pthread_mutex_lock(m); }
  static void ovm_mutex_unlock(ovm_mutex_t *m)   { pthread_mutex_unlock(m); }
  static int  ovm_mutex_trylock(ovm_mutex_t *m)  { return pthread_mutex_trylock(m); }
  static void ovm_cond_init(ovm_cond_t *c)       { pthread_cond_init(c, NULL); }
  static void ovm_cond_destroy(ovm_cond_t *c)    { pthread_cond_destroy(c); }
  static void ovm_cond_wait(ovm_cond_t *c, ovm_mutex_t *m) {
      pthread_cond_wait(c, m);
  }
  static void ovm_cond_signal(ovm_cond_t *c)     { pthread_cond_signal(c); }
  static void ovm_cond_broadcast(ovm_cond_t *c)  { pthread_cond_broadcast(c); }

  static void ovm_sleep_ms(int64_t ms) {
      struct timespec ts;
      ts.tv_sec  = ms / 1000;
      ts.tv_nsec = (ms % 1000) * 1000000L;
      nanosleep(&ts, NULL);
  }
  static void ovm_thread_yield_cpu(void) {
  #if defined(_POSIX_PRIORITY_SCHEDULING)
      sched_yield();
  #else
      usleep(0);
  #endif
  }

#endif

/* ─── Configuration ─────────────────────────────────────────────── */
#define STACK_MAX     65536
#define CALL_MAX      10000
#define NATIVE_MAX    128
#define HANDLE_MAX    1024
#define CHANNEL_CAP   256

/* ─── Opcodes (matches Rust OVM exactly) ────────────────────────── */
enum {
    /* Stack 0x00-0x0F */
    OP_NOP        = 0x00,
    OP_PUSH_I8    = 0x01,
    OP_PUSH_I16   = 0x02,
    OP_PUSH_I32   = 0x03,
    OP_PUSH_I64   = 0x04,
    OP_PUSH_F32   = 0x05,
    OP_PUSH_F64   = 0x06,
    OP_PUSH_STR   = 0x07,  /* LoadConst(u32) — load from const pool */
    OP_PUSH_NULL  = 0x08,
    OP_PUSH_TRUE  = 0x09,
    OP_PUSH_FALSE = 0x0A,
    OP_DUP        = 0x0B,
    OP_DUP2       = 0x0C,
    OP_SWAP       = 0x0D,
    OP_ROT        = 0x0E,
    OP_POP        = 0x0F,

    /* Arithmetic i64 0x10-0x17 */
    OP_ADD_I64    = 0x10,
    OP_SUB_I64    = 0x11,
    OP_MUL_I64    = 0x12,
    OP_DIV_I64    = 0x13,
    OP_MOD_I64    = 0x14,
    OP_NEG_I64    = 0x15,

    /* Arithmetic f64 0x18-0x1F */
    OP_ADD_F64    = 0x18,
    OP_SUB_F64    = 0x19,
    OP_MUL_F64    = 0x1A,
    OP_DIV_F64    = 0x1B,
    OP_NEG_F64    = 0x1C,

    /* Inc/Dec 0x20-0x21 */
    OP_INC        = 0x20,
    OP_DEC        = 0x21,

    /* Bitwise 0x30-0x35 */
    OP_BIT_AND    = 0x30,
    OP_BIT_OR     = 0x31,
    OP_BIT_XOR    = 0x32,
    OP_NOT        = 0x33,  /* Boolean NOT */
    OP_SHL        = 0x34,
    OP_SHR        = 0x35,

    /* Comparison 0x40-0x47 */
    OP_EQ         = 0x40,
    OP_NE         = 0x41,
    OP_LT         = 0x42,
    OP_LE         = 0x43,
    OP_GT         = 0x44,
    OP_GE         = 0x45,
    OP_CMP        = 0x46,
    OP_IS_NULL    = 0x47,

    /* Control 0x50-0x5C */
    OP_JMP        = 0x50,
    /* 0x51 reserved */
    OP_JZ         = 0x52,
    OP_JNZ        = 0x53,

    /* Calls 0x58-0x5B */
    OP_CALL       = 0x58,
    OP_CALL_IND   = 0x59,
    OP_RET        = 0x5A,
    OP_RET_VOID   = 0x5B,

    /* Locals 0x60-0x64 (u16 operand) */
    OP_LOAD_LOC   = 0x60,
    OP_STORE_LOC  = 0x61,
    /* 0x62-0x63 reserved */
    OP_ALLOC_LOC  = 0x64,

    /* Globals 0x70-0x72 */
    OP_LOAD_GLB   = 0x70,
    OP_STORE_GLB  = 0x71,
    OP_LOAD_CONST_POOL = 0x72,  /* load from const pool by u32 index */

    /* Heap 0x88-0x8C */
    OP_ALLOC      = 0x88,
    OP_REALLOC    = 0x89,
    OP_FREE       = 0x8A,
    OP_MEMCPY     = 0x8B,
    OP_MEMSET     = 0x8C,

    /* Objects 0x90-0x92 */
    OP_NEW        = 0x90,
    OP_GET_FIELD  = 0x91,
    OP_SET_FIELD  = 0x92,

    /* Arrays 0xA0-0xA4 */
    OP_NEW_ARRAY  = 0xA0,
    OP_ARRAY_LEN  = 0xA1,
    OP_ARRAY_GET  = 0xA2,
    OP_ARRAY_SET  = 0xA3,
    OP_ARRAY_SLICE = 0xA4,

    /* Conversion 0xB0-0xB3 */
    OP_I2F        = 0xB0,
    OP_F2I        = 0xB1,
    OP_I2B        = 0xB2,
    OP_B2I        = 0xB3,

    /* System 0xF0-0xFF */
    OP_SYSCALL    = 0xF0,
    OP_DEBUG      = 0xF1,
    OP_TRACE      = 0xF2,
    OP_ASSERT     = 0xF3,
    OP_HALT       = 0xFE,
    OP_PANIC      = 0xFF,
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
    VAL_OBJECT,
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

/* ─── Thread handle tracking ────────────────────────────────────── */
typedef struct {
    ovm_thread_t thread;
    int          active;
} ThreadSlot;

/* ─── Mutex handle tracking ─────────────────────────────────────── */
typedef struct {
    ovm_mutex_t  mutex;
    int          active;
} MutexSlot;

/* ─── Atomic value tracking ─────────────────────────────────────── */
typedef struct {
    atomic_int_fast64_t value;
    int                 active;
} AtomicSlot;

/* ─── Channel (bounded MPMC queue) ──────────────────────────────── */
typedef struct {
    Value       *buf;
    int          cap;
    int          head;
    int          tail;
    int          count;
    ovm_mutex_t  lock;
    ovm_cond_t   not_empty;
    ovm_cond_t   not_full;
    int          active;
} ChannelSlot;

/* ─── Global handle tables (shared across threads) ──────────────── */
static ThreadSlot   g_threads[HANDLE_MAX];
static int          g_thread_count;
static ovm_mutex_t  g_thread_lock;

static MutexSlot    g_mutexes[HANDLE_MAX];
static int          g_mutex_count;
static ovm_mutex_t  g_mutex_lock;

static AtomicSlot   g_atomics[HANDLE_MAX];
static int          g_atomic_count;
static ovm_mutex_t  g_atomic_lock;

static ChannelSlot  g_channels[HANDLE_MAX];
static int          g_channel_count;
static ovm_mutex_t  g_channel_lock;

static int g_handles_initialized = 0;

static void init_handle_tables(void) {
    if (g_handles_initialized) return;
    g_handles_initialized = 1;
    memset(g_threads,  0, sizeof(g_threads));
    memset(g_mutexes,  0, sizeof(g_mutexes));
    memset(g_atomics,  0, sizeof(g_atomics));
    memset(g_channels, 0, sizeof(g_channels));
    ovm_mutex_init(&g_thread_lock);
    ovm_mutex_init(&g_mutex_lock);
    ovm_mutex_init(&g_atomic_lock);
    ovm_mutex_init(&g_channel_lock);
}

/* ─── VM state ──────────────────────────────────────────────────── */
typedef struct OVM OVM;
struct OVM {
    /* Bytecode */
    uint8_t *code;
    uint64_t code_len;
    
    /* Constant pool */
    Value   *consts;
    uint32_t const_count;
    
    /* Symbol table: function name, code offset, param count */
    char    **func_names;
    uint64_t *func_addrs;
    uint16_t *func_params;
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
    
    /* Global locals (outside any frame) */
    Value    globals[256];
    
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
};

/* ─── String helpers ────────────────────────────────────────────── */
static char *ovm_strdup(OVM *vm, const char *s) {
    char *d = strdup(s);
    if (vm && vm->string_count < 4096)
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
        case VAL_NULL:   return 0;
        case VAL_BOOL:   return v.as.bool_val;
        case VAL_I64:    return v.as.i64 != 0;
        case VAL_F64:    return v.as.f64 != 0.0;
        case VAL_STRING: return v.as.str && v.as.str[0] != '\0';
        case VAL_ARRAY:  return v.as.arr.len > 0;
        default:         return 1;
    }
}

static int val_eq(Value a, Value b) {
    if (a.tag == VAL_I64 && b.tag == VAL_I64) return a.as.i64 == b.as.i64;
    if (a.tag == VAL_F64 && b.tag == VAL_F64) return fabs(a.as.f64 - b.as.f64) < 1e-15;
    if (a.tag == VAL_STRING && b.tag == VAL_STRING)
        return (a.as.str && b.as.str) ? strcmp(a.as.str, b.as.str) == 0 : a.as.str == b.as.str;
    if (a.tag == VAL_BOOL && b.tag == VAL_BOOL) return a.as.bool_val == b.as.bool_val;
    if (a.tag == VAL_NULL && b.tag == VAL_NULL) return 1;
    return 0;
}

static const char *val_type_name(Value v) {
    switch (v.tag) {
        case VAL_I64:    return "int";
        case VAL_F64:    return "float";
        case VAL_BOOL:   return "bool";
        case VAL_STRING: return "string";
        case VAL_ARRAY:  return "array";
        case VAL_OBJECT: return "object";
        case VAL_NULL:   return "null";
        default:         return "unknown";
    }
}

static void val_print(Value v) {
    switch (v.tag) {
        case VAL_I64:    printf("%lld", (long long)v.as.i64); break;
        case VAL_F64:    printf("%g", v.as.f64); break;
        case VAL_BOOL:   printf("%s", v.as.bool_val ? "true" : "false"); break;
        case VAL_STRING: printf("%s", v.as.str ? v.as.str : ""); break;
        case VAL_NULL:   printf("null"); break;
        default:         printf("<??>"); break;
    }
}

static char *val_to_cstr(Value v, char *buf, size_t bufsz) {
    switch (v.tag) {
        case VAL_I64:    snprintf(buf, bufsz, "%lld", (long long)v.as.i64); break;
        case VAL_F64:    snprintf(buf, bufsz, "%g", v.as.f64); break;
        case VAL_BOOL:   snprintf(buf, bufsz, "%s", v.as.bool_val ? "true" : "false"); break;
        case VAL_STRING: return v.as.str ? v.as.str : (char*)""; 
        case VAL_NULL:   snprintf(buf, bufsz, "null"); break;
        default:         snprintf(buf, bufsz, "<??>"); break;
    }
    return buf;
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
    int idx = vm->sp - 1 - offset;
    if (idx < 0) return v_null();
    return vm->stack[idx];
}

/* ─── Byte reading helpers ──────────────────────────────────────── */
static uint8_t  read_u8(OVM *vm)  { return vm->code[vm->pc++]; }
static uint16_t read_u16(OVM *vm) {
    uint16_t v = vm->code[vm->pc] | ((uint16_t)vm->code[vm->pc+1] << 8);
    vm->pc += 2;
    return v;
}
static uint32_t read_u32(OVM *vm) {
    uint32_t v = 0;
    memcpy(&v, &vm->code[vm->pc], 4);
    vm->pc += 4;
    return v;
}
static int32_t read_i32(OVM *vm) { return (int32_t)read_u32(vm); }
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
    for (int i = 0; i < nargs; i++) val_print(args[i]);
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
    char buf[256];
    return v_str(NULL, val_to_cstr(args[0], buf, sizeof(buf)));
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
    return v_str(NULL, val_type_name(args[0]));
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
    if (args[0].tag == VAL_STRING && args[0].as.str)
        return v_i64((int64_t)strlen(args[0].as.str));
    if (args[0].tag == VAL_ARRAY) return v_i64(args[0].as.arr.len);
    return v_i64(0);
}

static Value native_format(Value *args, int nargs) {
    if (nargs < 1) return v_str(NULL, "");
    char buf[256];
    return v_str(NULL, val_to_cstr(args[0], buf, sizeof(buf)));
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

static Value native_exit_fn(Value *args, int nargs) {
    int code = (nargs > 0) ? (int)val_as_int(args[0]) : 0;
    exit(code);
    return v_null(); /* unreachable */
}

static Value native_string_concat(Value *args, int nargs) {
    if (nargs < 2) return nargs > 0 ? args[0] : v_str(NULL, "");
    const char *a = (args[0].tag == VAL_STRING) ? args[0].as.str : "";
    const char *b = (args[1].tag == VAL_STRING) ? args[1].as.str : "";
    size_t la = strlen(a), lb = strlen(b);
    char *r = malloc(la + lb + 1);
    memcpy(r, a, la);
    memcpy(r + la, b, lb);
    r[la + lb] = '\0';
    return (Value){.tag = VAL_STRING, .as.str = r};
}

/* ─── Threading native functions ────────────────────────────────── */

/* Thread spawn argument — carries function index and a cloned OVM */
typedef struct {
    OVM     *vm;       /* cloned VM for thread */
    uint32_t func_idx; /* function to execute */
} ThreadArg;

static Value execute(OVM *vm); /* forward decl */

static void thread_body(ThreadArg *ta) {
    OVM *tvm = ta->vm;
    uint32_t func_idx = ta->func_idx;
    free(ta);

    if (func_idx < tvm->func_count) {
        tvm->pc = tvm->func_addrs[func_idx];
        tvm->running = 1;
        execute(tvm);
    }
    free(tvm);
}

#ifdef _WIN32
static DWORD WINAPI thread_entry_win(LPVOID arg) {
    thread_body((ThreadArg *)arg);
    return 0;
}
#else
static void *thread_entry_posix(void *arg) {
    thread_body((ThreadArg *)arg);
    return NULL;
}
#endif

static Value native_thread_spawn(Value *args, int nargs) {
    if (nargs < 1) return v_i64(-1);
    /* args[0] = function index, args[1] = source OVM pointer */
    int64_t func_idx = val_as_int(args[0]);
    OVM *src = (nargs >= 2 && args[1].tag == VAL_PTR) ? (OVM *)args[1].as.ptr : NULL;
    if (!src) return v_i64(-1);

    /* Clone VM for the thread */
    OVM *tvm = malloc(sizeof(OVM));
    memcpy(tvm, src, sizeof(OVM));
    tvm->sp = 0;
    tvm->fp = 0;
    tvm->pc = 0;

    ThreadArg *ta = malloc(sizeof(ThreadArg));
    ta->vm = tvm;
    ta->func_idx = (uint32_t)func_idx;

    ovm_mutex_lock(&g_thread_lock);
    int slot = -1;
    for (int i = 0; i < g_thread_count; i++) {
        if (!g_threads[i].active) { slot = i; break; }
    }
    if (slot < 0 && g_thread_count < HANDLE_MAX) {
        slot = g_thread_count++;
    }
    if (slot < 0) {
        ovm_mutex_unlock(&g_thread_lock);
        free(ta); free(tvm);
        return v_i64(-1);
    }
    g_threads[slot].active = 1;
#ifdef _WIN32
    int rc = ovm_thread_create(&g_threads[slot].thread,
                               thread_entry_win, ta);
#else
    int rc = ovm_thread_create(&g_threads[slot].thread,
                               thread_entry_posix, ta);
#endif
    if (rc != 0) {
        g_threads[slot].active = 0;
        ovm_mutex_unlock(&g_thread_lock);
        free(ta); free(tvm);
        return v_i64(-1);
    }
    ovm_mutex_unlock(&g_thread_lock);
    return v_i64(slot);
}

static Value native_thread_join(Value *args, int nargs) {
    if (nargs < 1) return v_i64(-1);
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX) return v_i64(-1);

    ovm_mutex_lock(&g_thread_lock);
    if (!g_threads[slot].active) {
        ovm_mutex_unlock(&g_thread_lock);
        return v_i64(-1);
    }
    ovm_thread_t t = g_threads[slot].thread;
    ovm_mutex_unlock(&g_thread_lock);

    ovm_thread_join(t);

    ovm_mutex_lock(&g_thread_lock);
    g_threads[slot].active = 0;
    ovm_mutex_unlock(&g_thread_lock);
    return v_i64(0);
}

static Value native_thread_sleep_ms(Value *args, int nargs) {
    if (nargs < 1) return v_null();
    int64_t ms = val_as_int(args[0]);
    if (ms > 0) ovm_sleep_ms(ms);
    return v_null();
}

static Value native_thread_yield(Value *args, int nargs) {
    (void)args; (void)nargs;
    ovm_thread_yield_cpu();
    return v_null();
}

/* ─── Mutex native functions ────────────────────────────────────── */

static Value native_mutex_new(Value *args, int nargs) {
    (void)args; (void)nargs;
    ovm_mutex_lock(&g_mutex_lock);
    int slot = -1;
    for (int i = 0; i < g_mutex_count; i++) {
        if (!g_mutexes[i].active) { slot = i; break; }
    }
    if (slot < 0 && g_mutex_count < HANDLE_MAX) {
        slot = g_mutex_count++;
    }
    if (slot < 0) {
        ovm_mutex_unlock(&g_mutex_lock);
        return v_i64(-1);
    }
    ovm_mutex_init(&g_mutexes[slot].mutex);
    g_mutexes[slot].active = 1;
    ovm_mutex_unlock(&g_mutex_lock);
    return v_i64(slot);
}

static Value native_mutex_lock(Value *args, int nargs) {
    if (nargs < 1) return v_i64(-1);
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_mutexes[slot].active) return v_i64(-1);
    ovm_mutex_lock(&g_mutexes[slot].mutex);
    return v_i64(0);
}

static Value native_mutex_unlock(Value *args, int nargs) {
    if (nargs < 1) return v_i64(-1);
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_mutexes[slot].active) return v_i64(-1);
    ovm_mutex_unlock(&g_mutexes[slot].mutex);
    return v_i64(0);
}

static Value native_mutex_try_lock(Value *args, int nargs) {
    if (nargs < 1) return v_bool(0);
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_mutexes[slot].active) return v_bool(0);
    return v_bool(ovm_mutex_trylock(&g_mutexes[slot].mutex) == 0);
}

/* ─── Atomic native functions ───────────────────────────────────── */

static Value native_atomic_new(Value *args, int nargs) {
    int64_t init = (nargs > 0) ? val_as_int(args[0]) : 0;
    ovm_mutex_lock(&g_atomic_lock);
    int slot = -1;
    for (int i = 0; i < g_atomic_count; i++) {
        if (!g_atomics[i].active) { slot = i; break; }
    }
    if (slot < 0 && g_atomic_count < HANDLE_MAX) {
        slot = g_atomic_count++;
    }
    if (slot < 0) {
        ovm_mutex_unlock(&g_atomic_lock);
        return v_i64(-1);
    }
    atomic_init(&g_atomics[slot].value, init);
    g_atomics[slot].active = 1;
    ovm_mutex_unlock(&g_atomic_lock);
    return v_i64(slot);
}

static Value native_atomic_load(Value *args, int nargs) {
    if (nargs < 1) return v_i64(0);
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_atomics[slot].active) return v_i64(0);
    return v_i64(atomic_load(&g_atomics[slot].value));
}

static Value native_atomic_store(Value *args, int nargs) {
    if (nargs < 2) return v_null();
    int slot = (int)val_as_int(args[0]);
    int64_t val = val_as_int(args[1]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_atomics[slot].active) return v_null();
    atomic_store(&g_atomics[slot].value, val);
    return v_null();
}

static Value native_atomic_cas(Value *args, int nargs) {
    if (nargs < 3) return v_bool(0);
    int slot = (int)val_as_int(args[0]);
    int64_t expected = val_as_int(args[1]);
    int64_t desired  = val_as_int(args[2]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_atomics[slot].active) return v_bool(0);
    int_fast64_t exp = expected;
    int ok = atomic_compare_exchange_strong(&g_atomics[slot].value, &exp, desired);
    return v_bool(ok);
}

static Value native_atomic_fetch_add(Value *args, int nargs) {
    if (nargs < 2) return v_i64(0);
    int slot = (int)val_as_int(args[0]);
    int64_t delta = val_as_int(args[1]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_atomics[slot].active) return v_i64(0);
    return v_i64(atomic_fetch_add(&g_atomics[slot].value, delta));
}

/* ─── Channel native functions ──────────────────────────────────── */

static Value native_channel_new(Value *args, int nargs) {
    int cap = (nargs > 0) ? (int)val_as_int(args[0]) : CHANNEL_CAP;
    if (cap <= 0) cap = CHANNEL_CAP;
    if (cap > 65536) cap = 65536;

    ovm_mutex_lock(&g_channel_lock);
    int slot = -1;
    for (int i = 0; i < g_channel_count; i++) {
        if (!g_channels[i].active) { slot = i; break; }
    }
    if (slot < 0 && g_channel_count < HANDLE_MAX) {
        slot = g_channel_count++;
    }
    if (slot < 0) {
        ovm_mutex_unlock(&g_channel_lock);
        return v_i64(-1);
    }
    ChannelSlot *ch = &g_channels[slot];
    ch->buf   = calloc((size_t)cap, sizeof(Value));
    ch->cap   = cap;
    ch->head  = 0;
    ch->tail  = 0;
    ch->count = 0;
    ovm_mutex_init(&ch->lock);
    ovm_cond_init(&ch->not_empty);
    ovm_cond_init(&ch->not_full);
    ch->active = 1;
    ovm_mutex_unlock(&g_channel_lock);
    return v_i64(slot);
}

static Value native_channel_send(Value *args, int nargs) {
    if (nargs < 2) return v_i64(-1);
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_channels[slot].active) return v_i64(-1);

    ChannelSlot *ch = &g_channels[slot];
    ovm_mutex_lock(&ch->lock);
    while (ch->count >= ch->cap) {
        ovm_cond_wait(&ch->not_full, &ch->lock);
    }
    ch->buf[ch->tail] = args[1];
    ch->tail = (ch->tail + 1) % ch->cap;
    ch->count++;
    ovm_cond_signal(&ch->not_empty);
    ovm_mutex_unlock(&ch->lock);
    return v_i64(0);
}

static Value native_channel_recv(Value *args, int nargs) {
    if (nargs < 1) return v_null();
    int slot = (int)val_as_int(args[0]);
    if (slot < 0 || slot >= HANDLE_MAX || !g_channels[slot].active) return v_null();

    ChannelSlot *ch = &g_channels[slot];
    ovm_mutex_lock(&ch->lock);
    while (ch->count == 0) {
        ovm_cond_wait(&ch->not_empty, &ch->lock);
    }
    Value val = ch->buf[ch->head];
    ch->head = (ch->head + 1) % ch->cap;
    ch->count--;
    ovm_cond_signal(&ch->not_full);
    ovm_mutex_unlock(&ch->lock);
    return val;
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
    if (const_off > 0 && const_len > 0 && const_off < size) {
        const uint8_t *cp = data + const_off;
        uint32_t count;
        memcpy(&count, cp, 4);
        cp += 4;
        
        if (count > 1000000) {
            fprintf(stderr, "OVM: Too many constants (%u)\n", count);
            return -1;
        }
        
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
    
    /* Load code section (contains function table + bytecode) */
    if (code_off > 0 && code_len > 0 && code_off + code_len <= size) {
        const uint8_t *cd = data + code_off;
        uint32_t nfuncs;
        memcpy(&nfuncs, cd, 4);
        
        if (nfuncs > 1000000) {
            fprintf(stderr, "OVM: Too many functions (%u)\n", nfuncs);
            return -1;
        }
        
        vm->func_count = nfuncs;
        vm->func_names  = calloc(nfuncs, sizeof(char*));
        vm->func_addrs  = calloc(nfuncs, sizeof(uint64_t));
        vm->func_params = calloc(nfuncs, sizeof(uint16_t));
        
        /* Build code by concatenating function bodies */
        uint8_t *all_code = malloc(code_len);
        size_t code_pos = 0;
        
        size_t p = 4;
        for (uint32_t i = 0; i < nfuncs; i++) {
            if (p + 14 > code_len) break;
            
            uint32_t name_idx;
            memcpy(&name_idx, cd + p, 4);
            p += 4;
            
            uint16_t param_count;
            memcpy(&param_count, cd + p, 2);
            p += 2;
            p += 2; /* local_count */
            p += 2; /* stack_size */
            
            uint32_t body_len;
            memcpy(&body_len, cd + p, 4);
            p += 4;
            
            /* Resolve name from const pool */
            char namebuf[64];
            if (name_idx < vm->const_count && vm->consts[name_idx].tag == VAL_STRING) {
                vm->func_names[i] = strdup(vm->consts[name_idx].as.str);
            } else {
                snprintf(namebuf, sizeof(namebuf), "f%u", name_idx);
                vm->func_names[i] = strdup(namebuf);
            }
            
            vm->func_addrs[i]  = code_pos;
            vm->func_params[i] = param_count;
            
            if (p + body_len <= code_len) {
                memcpy(all_code + code_pos, cd + p, body_len);
                code_pos += body_len;
                p += body_len;
            }
        }
        
        vm->code = all_code;
        vm->code_len = code_pos;
    }
    
    /* Load symbol table (legacy format) if code section didn't have functions */
    if (vm->func_count == 0) {
        uint64_t sym_off, sym_len;
        memcpy(&sym_off, data + 56, 8);
        memcpy(&sym_len, data + 64, 8);
        
        if (sym_off > 0 && sym_len > 0 && sym_off + sym_len <= size) {
            const uint8_t *sp = data + sym_off;
            uint32_t count;
            memcpy(&count, sp, 4);
            sp += 4;
            
            vm->func_count  = count;
            vm->func_names  = calloc(count, sizeof(char*));
            vm->func_addrs  = calloc(count, sizeof(uint64_t));
            vm->func_params = calloc(count, sizeof(uint16_t));
            
            for (uint32_t i = 0; i < count; i++) {
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
                
                vm->func_names[i]  = name;
                vm->func_addrs[i]  = addr;
                vm->func_params[i] = 0;
            }
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

/* ─── Dispatch a named syscall (matches Rust OVM syscall handler) ─ */
static void dispatch_syscall(OVM *vm, const char *fname) {
    /* Strip module prefix (e.g., "core::println" → "println") */
    const char *func_name = fname;
    const char *last_colon = strrchr(fname, ':');
    if (last_colon && last_colon > fname && *(last_colon - 1) == ':') {
        func_name = last_colon + 1;
    }

    if (strcmp(func_name, "print") == 0) {
        Value v = pop(vm);
        val_print(v);
        push(vm, v_null());
    } else if (strcmp(func_name, "println") == 0) {
        Value v = pop(vm);
        val_print(v);
        printf("\n");
        fflush(stdout);
        push(vm, v_null());
    } else if (strcmp(func_name, "format") == 0 ||
               strcmp(func_name, "to_string") == 0 ||
               strcmp(func_name, "stringify") == 0 ||
               strcmp(func_name, "str") == 0) {
        Value v = pop(vm);
        char buf[256];
        push(vm, v_str(vm, val_to_cstr(v, buf, sizeof(buf))));
    } else if (strcmp(func_name, "type_of") == 0) {
        Value v = pop(vm);
        push(vm, v_str(vm, val_type_name(v)));
    } else if (strcmp(func_name, "len") == 0) {
        Value v = pop(vm);
        int64_t l = 0;
        if (v.tag == VAL_STRING && v.as.str) l = (int64_t)strlen(v.as.str);
        else if (v.tag == VAL_ARRAY) l = v.as.arr.len;
        push(vm, v_i64(l));
    } else if (strcmp(func_name, "assert") == 0) {
        Value v = pop(vm);
        if (!val_truthy(v)) {
            fprintf(stderr, "Assertion failed\n");
            exit(1);
        }
        push(vm, v_null());
    } else if (strcmp(func_name, "int") == 0 || strcmp(func_name, "to_int") == 0) {
        Value v = pop(vm);
        push(vm, v_i64(val_as_int(v)));
    } else if (strcmp(func_name, "float") == 0 || strcmp(func_name, "to_float") == 0) {
        Value v = pop(vm);
        push(vm, v_f64(val_as_float(v)));
    } else if (strcmp(func_name, "sqrt") == 0) {
        Value v = pop(vm);
        push(vm, v_f64(sqrt(val_as_float(v))));
    } else if (strcmp(func_name, "abs") == 0) {
        Value v = pop(vm);
        push(vm, v_i64(val_as_int(v) < 0 ? -val_as_int(v) : val_as_int(v)));
    } else if (strcmp(func_name, "args") == 0 || strcmp(func_name, "argv") == 0) {
        /* Build array of arg strings */
        Value arr = {.tag = VAL_ARRAY};
        arr.as.arr.len = vm->argc;
        arr.as.arr.cap = vm->argc > 0 ? vm->argc : 1;
        arr.as.arr.items = calloc((size_t)arr.as.arr.cap, sizeof(Value));
        for (int i = 0; i < vm->argc; i++) {
            arr.as.arr.items[i] = v_str(vm, vm->argv[i]);
        }
        push(vm, arr);
    } else if (strcmp(func_name, "arg_count") == 0 || strcmp(func_name, "argc") == 0) {
        push(vm, v_i64(vm->argc));
    } else if (strcmp(func_name, "arg_at") == 0) {
        Value idx = pop(vm);
        int i = (int)val_as_int(idx);
        if (i >= 0 && i < vm->argc)
            push(vm, v_str(vm, vm->argv[i]));
        else
            push(vm, v_null());
    /* ── Threading syscalls ── */
    } else if (strcmp(func_name, "thread_spawn") == 0) {
        Value fidx = pop(vm);
        Value vmptr = {.tag = VAL_PTR, .as.ptr = vm};
        Value targs[2] = {fidx, vmptr};
        push(vm, native_thread_spawn(targs, 2));
    } else if (strcmp(func_name, "thread_join") == 0) {
        Value handle = pop(vm);
        push(vm, native_thread_join(&handle, 1));
    } else if (strcmp(func_name, "thread_sleep_ms") == 0) {
        Value ms = pop(vm);
        native_thread_sleep_ms(&ms, 1);
        push(vm, v_null());
    } else if (strcmp(func_name, "thread_yield") == 0) {
        native_thread_yield(NULL, 0);
        push(vm, v_null());
    /* ── Mutex syscalls ── */
    } else if (strcmp(func_name, "mutex_new") == 0) {
        push(vm, native_mutex_new(NULL, 0));
    } else if (strcmp(func_name, "mutex_lock") == 0) {
        Value h = pop(vm);
        push(vm, native_mutex_lock(&h, 1));
    } else if (strcmp(func_name, "mutex_unlock") == 0) {
        Value h = pop(vm);
        push(vm, native_mutex_unlock(&h, 1));
    } else if (strcmp(func_name, "mutex_try_lock") == 0) {
        Value h = pop(vm);
        push(vm, native_mutex_try_lock(&h, 1));
    /* ── Atomic syscalls ── */
    } else if (strcmp(func_name, "atomic_new") == 0) {
        Value init = pop(vm);
        push(vm, native_atomic_new(&init, 1));
    } else if (strcmp(func_name, "atomic_load") == 0) {
        Value h = pop(vm);
        push(vm, native_atomic_load(&h, 1));
    } else if (strcmp(func_name, "atomic_store") == 0) {
        Value val = pop(vm);
        Value h = pop(vm);
        Value aargs[2] = {h, val};
        native_atomic_store(aargs, 2);
        push(vm, v_null());
    } else if (strcmp(func_name, "atomic_cas") == 0) {
        Value desired = pop(vm);
        Value expected = pop(vm);
        Value h = pop(vm);
        Value aargs[3] = {h, expected, desired};
        push(vm, native_atomic_cas(aargs, 3));
    } else if (strcmp(func_name, "atomic_fetch_add") == 0) {
        Value delta = pop(vm);
        Value h = pop(vm);
        Value aargs[2] = {h, delta};
        push(vm, native_atomic_fetch_add(aargs, 2));
    /* ── Channel syscalls ── */
    } else if (strcmp(func_name, "channel_new") == 0) {
        Value cap = pop(vm);
        push(vm, native_channel_new(&cap, 1));
    } else if (strcmp(func_name, "channel_send") == 0) {
        Value val = pop(vm);
        Value h = pop(vm);
        Value cargs[2] = {h, val};
        push(vm, native_channel_send(cargs, 2));
    } else if (strcmp(func_name, "channel_recv") == 0) {
        Value h = pop(vm);
        push(vm, native_channel_recv(&h, 1));
    /* ── File I/O stubs ── */
    } else if (strcmp(func_name, "open") == 0 ||
               strcmp(func_name, "read_file") == 0 ||
               strcmp(func_name, "write_file") == 0 ||
               strcmp(func_name, "close") == 0) {
        pop(vm); /* consume argument */
        fprintf(stderr, "OVM: syscall '%s' not implemented\n", func_name);
        push(vm, v_null());
    } else {
        /* Unknown syscall — push null */
        push(vm, v_null());
    }
}

/* ─── Dispatch a named function call (for OP_CALL) ──────────────── */
static int dispatch_builtin_call(OVM *vm, const char *fname) {
    /* Returns 1 if handled as builtin, 0 otherwise */
    if (strcmp(fname, "print") == 0 || strcmp(fname, "println") == 0) {
        Value v = pop(vm);
        val_print(v);
        if (strcmp(fname, "println") == 0) { printf("\n"); fflush(stdout); }
        push(vm, v_null());
        return 1;
    }
    if (strcmp(fname, "format") == 0 || strcmp(fname, "to_string") == 0 ||
        strcmp(fname, "str") == 0) {
        Value v = pop(vm);
        char buf[256];
        push(vm, v_str(vm, val_to_cstr(v, buf, sizeof(buf))));
        return 1;
    }
    if (strcmp(fname, "int") == 0) {
        Value v = pop(vm); push(vm, v_i64(val_as_int(v))); return 1;
    }
    if (strcmp(fname, "float") == 0) {
        Value v = pop(vm); push(vm, v_f64(val_as_float(v))); return 1;
    }
    if (strcmp(fname, "type_of") == 0) {
        Value v = pop(vm); push(vm, v_str(vm, val_type_name(v))); return 1;
    }
    if (strcmp(fname, "len") == 0) {
        Value v = pop(vm);
        int64_t l = 0;
        if (v.tag == VAL_STRING && v.as.str) l = (int64_t)strlen(v.as.str);
        else if (v.tag == VAL_ARRAY) l = v.as.arr.len;
        push(vm, v_i64(l));
        return 1;
    }
    if (strcmp(fname, "assert") == 0) {
        Value v = pop(vm);
        if (!val_truthy(v)) { fprintf(stderr, "Assertion failed\n"); exit(1); }
        push(vm, v_null());
        return 1;
    }
    if (strcmp(fname, "sqrt") == 0) {
        Value v = pop(vm); push(vm, v_f64(sqrt(val_as_float(v)))); return 1;
    }
    if (strcmp(fname, "abs") == 0) {
        Value v = pop(vm); push(vm, v_i64(val_as_int(v) < 0 ? -val_as_int(v) : val_as_int(v))); return 1;
    }
    if (strcmp(fname, "args") == 0 || strcmp(fname, "argv") == 0) {
        Value arr = {.tag = VAL_ARRAY};
        arr.as.arr.len = vm->argc;
        arr.as.arr.cap = vm->argc > 0 ? vm->argc : 1;
        arr.as.arr.items = calloc((size_t)arr.as.arr.cap, sizeof(Value));
        for (int i = 0; i < vm->argc; i++)
            arr.as.arr.items[i] = v_str(vm, vm->argv[i]);
        push(vm, arr);
        return 1;
    }
    if (strcmp(fname, "arg_count") == 0 || strcmp(fname, "argc") == 0) {
        push(vm, v_i64(vm->argc)); return 1;
    }
    if (strcmp(fname, "arg_at") == 0) {
        Value idx = pop(vm);
        int i = (int)val_as_int(idx);
        push(vm, (i >= 0 && i < vm->argc) ? v_str(vm, vm->argv[i]) : v_null());
        return 1;
    }
    /* ── File I/O stubs ── */
    if (strcmp(fname, "open") == 0 || strcmp(fname, "read_file") == 0 ||
        strcmp(fname, "write_file") == 0 || strcmp(fname, "close") == 0) {
        pop(vm);
        fprintf(stderr, "OVM: '%s' not implemented\n", fname);
        push(vm, v_null());
        return 1;
    }
    return 0; /* not a builtin */
}

/* ─── Main execution loop ───────────────────────────────────────── */
static Value execute(OVM *vm) {
    while (vm->running && vm->pc < vm->code_len) {
        uint8_t op = vm->code[vm->pc++];
        
        switch (op) {
            /* ── NOP ── */
            case OP_NOP: break;

            /* ── Push constants ── */
            case OP_PUSH_I8:  push(vm, v_i64((int8_t)read_u8(vm))); break;
            case OP_PUSH_I16: push(vm, v_i64((int16_t)read_u16(vm))); break;
            case OP_PUSH_I32: push(vm, v_i64((int32_t)read_u32(vm))); break;
            case OP_PUSH_I64: push(vm, v_i64(read_i64(vm))); break;
            case OP_PUSH_F64: push(vm, v_f64(read_f64(vm))); break;
            case OP_PUSH_STR: { /* LoadConst(u32) — load from const pool */
                uint32_t idx = read_u32(vm);
                push(vm, (idx < vm->const_count) ? vm->consts[idx] : v_null());
                break;
            }
            case OP_PUSH_NULL:  push(vm, v_null()); break;
            case OP_PUSH_TRUE:  push(vm, v_bool(1)); break;
            case OP_PUSH_FALSE: push(vm, v_bool(0)); break;
            case OP_DUP: { Value v = peek(vm, 0); push(vm, v); break; }
            case OP_DUP2: {
                Value a = peek(vm, 1), b = peek(vm, 0);
                push(vm, a); push(vm, b);
                break;
            }
            case OP_SWAP: {
                Value a = pop(vm), b = pop(vm);
                push(vm, a); push(vm, b);
                break;
            }
            case OP_ROT: {
                /* Rotate top 3: [a, b, c] → [b, c, a] */
                Value c = pop(vm), b = pop(vm), a = pop(vm);
                push(vm, b); push(vm, c); push(vm, a);
                break;
            }
            case OP_POP: pop(vm); break;
            
            /* ── Arithmetic (i64) ── */
            case OP_ADD_I64: {
                Value lv = pop(vm), rv = pop(vm);
                /* String concatenation support (like Rust OVM) */
                if (lv.tag == VAL_STRING || rv.tag == VAL_STRING) {
                    char lbuf[256], rbuf[256];
                    const char *ls = val_to_cstr(lv, lbuf, sizeof(lbuf));
                    const char *rs = val_to_cstr(rv, rbuf, sizeof(rbuf));
                    size_t ll = strlen(ls), rl = strlen(rs);
                    char *r = malloc(ll + rl + 1);
                    memcpy(r, ls, ll);
                    memcpy(r + ll, rs, rl);
                    r[ll + rl] = '\0';
                    push(vm, (Value){.tag = VAL_STRING, .as.str = r});
                } else if (lv.tag == VAL_F64 || rv.tag == VAL_F64) {
                    push(vm, v_f64(val_as_float(lv) + val_as_float(rv)));
                } else {
                    push(vm, v_i64(val_as_int(lv) + val_as_int(rv)));
                }
                break;
            }
            case OP_SUB_I64: {
                Value lv = pop(vm), rv = pop(vm);
                if (lv.tag == VAL_F64 || rv.tag == VAL_F64)
                    push(vm, v_f64(val_as_float(lv) - val_as_float(rv)));
                else
                    push(vm, v_i64(val_as_int(lv) - val_as_int(rv)));
                break;
            }
            case OP_MUL_I64: {
                Value lv = pop(vm), rv = pop(vm);
                if (lv.tag == VAL_F64 || rv.tag == VAL_F64)
                    push(vm, v_f64(val_as_float(lv) * val_as_float(rv)));
                else
                    push(vm, v_i64(val_as_int(lv) * val_as_int(rv)));
                break;
            }
            case OP_DIV_I64: {
                Value lv = pop(vm), rv = pop(vm);
                if (lv.tag == VAL_F64 || rv.tag == VAL_F64) {
                    push(vm, v_f64(val_as_float(lv) / val_as_float(rv)));
                } else {
                    int64_t r = val_as_int(rv), l = val_as_int(lv);
                    if (r == 0) { fprintf(stderr, "Division by zero\n"); exit(1); }
                    push(vm, v_i64(l / r));
                }
                break;
            }
            case OP_MOD_I64: {
                int64_t l = val_as_int(pop(vm)), r = val_as_int(pop(vm));
                if (r == 0) { fprintf(stderr, "Modulo by zero\n"); exit(1); }
                push(vm, v_i64(l % r));
                break;
            }
            case OP_NEG_I64: { push(vm, v_i64(-val_as_int(pop(vm)))); break; }
            
            /* ── Arithmetic (f64) ── */
            case OP_ADD_F64: { double l = val_as_float(pop(vm)), r = val_as_float(pop(vm)); push(vm, v_f64(l + r)); break; }
            case OP_SUB_F64: { double l = val_as_float(pop(vm)), r = val_as_float(pop(vm)); push(vm, v_f64(l - r)); break; }
            case OP_MUL_F64: { double l = val_as_float(pop(vm)), r = val_as_float(pop(vm)); push(vm, v_f64(l * r)); break; }
            case OP_DIV_F64: { double l = val_as_float(pop(vm)), r = val_as_float(pop(vm)); push(vm, v_f64(l / r)); break; }
            case OP_NEG_F64: { push(vm, v_f64(-val_as_float(pop(vm)))); break; }
            
            /* ── Inc/Dec ── */
            case OP_INC: { push(vm, v_i64(val_as_int(pop(vm)) + 1)); break; }
            case OP_DEC: { push(vm, v_i64(val_as_int(pop(vm)) - 1)); break; }
            
            /* ── Bitwise ── */
            case OP_BIT_AND: { int64_t l = val_as_int(pop(vm)), r = val_as_int(pop(vm)); push(vm, v_i64(l & r)); break; }
            case OP_BIT_OR:  { int64_t l = val_as_int(pop(vm)), r = val_as_int(pop(vm)); push(vm, v_i64(l | r)); break; }
            case OP_BIT_XOR: { int64_t l = val_as_int(pop(vm)), r = val_as_int(pop(vm)); push(vm, v_i64(l ^ r)); break; }
            case OP_NOT:     { push(vm, v_bool(!val_truthy(pop(vm)))); break; }
            case OP_SHL:     { int64_t l = val_as_int(pop(vm)), r = val_as_int(pop(vm)); push(vm, v_i64(l << r)); break; }
            case OP_SHR:     { int64_t l = val_as_int(pop(vm)), r = val_as_int(pop(vm)); push(vm, v_i64(l >> r)); break; }
            
            /* ── Comparison ── */
            case OP_EQ: {
                Value l = pop(vm), r = pop(vm);
                push(vm, v_bool(val_eq(l, r)));
                break;
            }
            case OP_NE: {
                Value l = pop(vm), r = pop(vm);
                push(vm, v_bool(!val_eq(l, r)));
                break;
            }
            case OP_LT: {
                Value l = pop(vm), r = pop(vm);
                if (l.tag == VAL_I64 && r.tag == VAL_I64) push(vm, v_bool(l.as.i64 < r.as.i64));
                else push(vm, v_bool(val_as_float(l) < val_as_float(r)));
                break;
            }
            case OP_LE: {
                Value l = pop(vm), r = pop(vm);
                if (l.tag == VAL_I64 && r.tag == VAL_I64) push(vm, v_bool(l.as.i64 <= r.as.i64));
                else push(vm, v_bool(val_as_float(l) <= val_as_float(r)));
                break;
            }
            case OP_GT: {
                Value l = pop(vm), r = pop(vm);
                if (l.tag == VAL_I64 && r.tag == VAL_I64) push(vm, v_bool(l.as.i64 > r.as.i64));
                else push(vm, v_bool(val_as_float(l) > val_as_float(r)));
                break;
            }
            case OP_GE: {
                Value l = pop(vm), r = pop(vm);
                if (l.tag == VAL_I64 && r.tag == VAL_I64) push(vm, v_bool(l.as.i64 >= r.as.i64));
                else push(vm, v_bool(val_as_float(l) >= val_as_float(r)));
                break;
            }
            case OP_CMP: {
                /* Three-way comparison → -1, 0, 1 */
                Value l = pop(vm), r = pop(vm);
                int result;
                if (l.tag == VAL_I64 && r.tag == VAL_I64) {
                    result = (l.as.i64 < r.as.i64) ? -1 : (l.as.i64 > r.as.i64) ? 1 : 0;
                } else {
                    double lf = val_as_float(l), rf = val_as_float(r);
                    result = (lf < rf) ? -1 : (lf > rf) ? 1 : 0;
                }
                push(vm, v_i64(result));
                break;
            }
            case OP_IS_NULL: {
                push(vm, v_bool(pop(vm).tag == VAL_NULL));
                break;
            }
            
            /* ── Jumps ── */
            case OP_JMP: {
                int32_t offset = read_i32(vm);
                vm->pc = (uint64_t)((int64_t)vm->pc + offset);
                break;
            }
            case OP_JZ: {
                int32_t off = read_i32(vm);
                if (!val_truthy(pop(vm)))
                    vm->pc = (uint64_t)((int64_t)vm->pc + off);
                break;
            }
            case OP_JNZ: {
                int32_t off = read_i32(vm);
                if (val_truthy(pop(vm)))
                    vm->pc = (uint64_t)((int64_t)vm->pc + off);
                break;
            }
            
            /* ── Calls ── */
            case OP_CALL: {
                uint32_t func_idx = read_u32(vm);
                if (func_idx >= vm->func_count) {
                    fprintf(stderr, "OVM: Invalid function index %u (have %u)\n",
                            func_idx, vm->func_count);
                    exit(1);
                }
                const char *fname = vm->func_names[func_idx];
                
                /* Try builtin dispatch first */
                if (dispatch_builtin_call(vm, fname)) break;
                
                /* Stack overflow protection */
                if (vm->fp >= CALL_MAX) {
                    fprintf(stderr, "OVM: Call stack overflow (max %d)\n", CALL_MAX);
                    exit(1);
                }
                
                uint16_t param_count = vm->func_params[func_idx];
                
                /* Pop args from stack */
                Value args[256];
                int nargs = (param_count <= 256) ? param_count : 256;
                for (int i = nargs - 1; i >= 0; i--) {
                    args[i] = pop(vm);
                }
                
                /* Push call frame */
                vm->frames[vm->fp].return_pc = vm->pc;
                vm->frames[vm->fp].stack_base = vm->sp;
                memset(vm->frames[vm->fp].locals, 0, sizeof(vm->frames[vm->fp].locals));
                
                /* Store args in callee's locals */
                for (int i = 0; i < nargs; i++) {
                    vm->frames[vm->fp].locals[i] = args[i];
                }
                
                vm->fp++;
                vm->pc = vm->func_addrs[func_idx];
                break;
            }
            case OP_CALL_IND: {
                Value fv = pop(vm);
                uint32_t func_idx = (uint32_t)val_as_int(fv);
                if (func_idx >= vm->func_count) {
                    push(vm, v_null());
                    break;
                }
                const char *fname = vm->func_names[func_idx];
                if (dispatch_builtin_call(vm, fname)) break;

                if (vm->fp >= CALL_MAX) {
                    fprintf(stderr, "OVM: Call stack overflow\n");
                    exit(1);
                }
                
                uint16_t param_count = vm->func_params[func_idx];
                Value args[256];
                int nargs = (param_count <= 256) ? param_count : 256;
                for (int i = nargs - 1; i >= 0; i--)
                    args[i] = pop(vm);
                
                vm->frames[vm->fp].return_pc = vm->pc;
                vm->frames[vm->fp].stack_base = vm->sp;
                memset(vm->frames[vm->fp].locals, 0, sizeof(vm->frames[vm->fp].locals));
                for (int i = 0; i < nargs; i++)
                    vm->frames[vm->fp].locals[i] = args[i];
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
            
            /* ── Locals (u16 operand — matches Rust OVM) ── */
            case OP_LOAD_LOC: {
                uint16_t idx = read_u16(vm);
                if (vm->fp > 0 && idx < 256)
                    push(vm, vm->frames[vm->fp - 1].locals[idx]);
                else if (vm->fp == 0 && idx < 256)
                    push(vm, vm->globals[idx]);
                else
                    push(vm, v_null());
                break;
            }
            case OP_STORE_LOC: {
                uint16_t idx = read_u16(vm);
                Value val = pop(vm);
                if (vm->fp > 0 && idx < 256)
                    vm->frames[vm->fp - 1].locals[idx] = val;
                else if (vm->fp == 0 && idx < 256)
                    vm->globals[idx] = val;
                break;
            }
            case OP_ALLOC_LOC: {
                /* No-op — just skip the operand */
                read_u16(vm);
                break;
            }
            
            /* ── Globals ── */
            case OP_LOAD_GLB: {
                read_u32(vm); /* index — stub */
                push(vm, v_null());
                break;
            }
            case OP_STORE_GLB: {
                read_u32(vm);
                pop(vm);
                break;
            }
            case OP_LOAD_CONST_POOL: {
                uint32_t idx = read_u32(vm);
                push(vm, (idx < vm->const_count) ? vm->consts[idx] : v_null());
                break;
            }
            
            /* ── Heap ── */
            case OP_ALLOC: {
                uint32_t size = read_u32(vm);
                void *p = calloc(1, size > 0 ? size : 1);
                push(vm, (Value){.tag = VAL_PTR, .as.ptr = p});
                break;
            }
            case OP_REALLOC: {
                uint32_t new_size = read_u32(vm);
                Value v = pop(vm);
                if (v.tag == VAL_PTR && v.as.ptr) {
                    v.as.ptr = realloc(v.as.ptr, new_size > 0 ? new_size : 1);
                }
                push(vm, v);
                break;
            }
            case OP_FREE: {
                Value v = pop(vm);
                if (v.tag == VAL_PTR && v.as.ptr) free(v.as.ptr);
                break;
            }
            case OP_MEMCPY: {
                Value sz = pop(vm), src = pop(vm), dst = pop(vm);
                (void)sz; (void)src; (void)dst;
                /* Simplified — would need proper ptr tracking */
                break;
            }
            case OP_MEMSET: {
                Value sz = pop(vm), val = pop(vm), ptr = pop(vm);
                (void)sz; (void)val; (void)ptr;
                break;
            }
            
            /* ── Objects ── */
            case OP_NEW: {
                uint32_t type_idx = read_u32(vm);
                uint16_t field_count = read_u16(vm);
                (void)type_idx;
                /* Pop field_count * 2 values (name, value pairs) */
                for (uint16_t i = 0; i < field_count * 2; i++) pop(vm);
                push(vm, (Value){.tag = VAL_OBJECT, .as.ptr = NULL});
                break;
            }
            case OP_GET_FIELD: {
                uint32_t field_idx = read_u32(vm);
                pop(vm); /* object */
                (void)field_idx;
                push(vm, v_null()); /* stub */
                break;
            }
            case OP_SET_FIELD: {
                uint32_t field_idx = read_u32(vm);
                Value val = pop(vm);
                (void)field_idx; (void)val;
                pop(vm); /* object — leave it consumed */
                break;
            }
            
            /* ── Arrays ── */
            case OP_NEW_ARRAY: {
                uint32_t count = read_u32(vm);
                Value arr = {.tag = VAL_ARRAY};
                arr.as.arr.len = (int)count;
                arr.as.arr.cap = (int)(count > 0 ? count : 8);
                arr.as.arr.items = calloc((size_t)arr.as.arr.cap, sizeof(Value));
                /* Pop elements from stack (they were pushed before) */
                for (int i = (int)count - 1; i >= 0; i--) {
                    arr.as.arr.items[i] = pop(vm);
                }
                push(vm, arr);
                break;
            }
            case OP_ARRAY_LEN: {
                Value v = pop(vm);
                if (v.tag == VAL_ARRAY) push(vm, v_i64(v.as.arr.len));
                else if (v.tag == VAL_STRING && v.as.str) push(vm, v_i64((int64_t)strlen(v.as.str)));
                else push(vm, v_i64(0));
                break;
            }
            case OP_ARRAY_GET: {
                Value idx = pop(vm), arr = pop(vm);
                int i = (int)val_as_int(idx);
                if (arr.tag == VAL_ARRAY && i >= 0 && i < arr.as.arr.len)
                    push(vm, arr.as.arr.items[i]);
                else if (arr.tag == VAL_STRING && arr.as.str && i >= 0 && i < (int)strlen(arr.as.str))
                    push(vm, v_i64((int64_t)(unsigned char)arr.as.str[i]));
                else
                    push(vm, v_null());
                break;
            }
            case OP_ARRAY_SET: {
                Value val = pop(vm), idx = pop(vm);
                /* Peek at array on stack, modify in place, then pop */
                if (vm->sp > 0) {
                    Value *arrp = &vm->stack[vm->sp - 1];
                    int i = (int)val_as_int(idx);
                    if (arrp->tag == VAL_ARRAY && i >= 0 && i < arrp->as.arr.len)
                        arrp->as.arr.items[i] = val;
                }
                pop(vm); /* pop the array */
                break;
            }
            case OP_ARRAY_SLICE: {
                Value end = pop(vm), start = pop(vm), arr = pop(vm);
                int s = (int)val_as_int(start), e = (int)val_as_int(end);
                if (arr.tag == VAL_ARRAY) {
                    if (e > arr.as.arr.len) e = arr.as.arr.len;
                    if (s < 0) s = 0;
                    int len = (e > s) ? e - s : 0;
                    Value slice = {.tag = VAL_ARRAY};
                    slice.as.arr.len = len;
                    slice.as.arr.cap = len > 0 ? len : 1;
                    slice.as.arr.items = calloc((size_t)slice.as.arr.cap, sizeof(Value));
                    for (int i = 0; i < len; i++)
                        slice.as.arr.items[i] = arr.as.arr.items[s + i];
                    push(vm, slice);
                } else {
                    Value empty = {.tag = VAL_ARRAY};
                    empty.as.arr.len = 0;
                    empty.as.arr.cap = 1;
                    empty.as.arr.items = calloc(1, sizeof(Value));
                    push(vm, empty);
                }
                break;
            }
            
            /* ── Conversion ── */
            case OP_I2F: { push(vm, v_f64((double)val_as_int(pop(vm)))); break; }
            case OP_F2I: { push(vm, v_i64((int64_t)val_as_float(pop(vm)))); break; }
            case OP_I2B: { push(vm, v_bool(val_truthy(pop(vm)))); break; }
            case OP_B2I: { Value v = pop(vm); push(vm, v_i64(val_truthy(v) ? 1 : 0)); break; }
            
            /* ── System calls (u16 const index — matches Rust OVM) ── */
            case OP_SYSCALL: {
                uint16_t name_idx = read_u16(vm);
                if (name_idx < vm->const_count && vm->consts[name_idx].tag == VAL_STRING) {
                    dispatch_syscall(vm, vm->consts[name_idx].as.str);
                } else {
                    /* Fallback: treat as numeric syscall */
                    push(vm, v_null());
                }
                break;
            }
            
            /* ── Debug/Trace/Assert ── */
            case OP_DEBUG: break;
            case OP_TRACE: break;
            case OP_ASSERT: break;
            
            /* ── Control ── */
            case OP_HALT:  vm->running = 0; return v_null();
            case OP_PANIC: {
                Value msg = pop(vm);
                char buf[256];
                fprintf(stderr, "Panic: %s\n", val_to_cstr(msg, buf, sizeof(buf)));
                exit(1);
            }
            
            default:
                fprintf(stderr, "OVM: Unknown opcode 0x%02X at pc %llu\n",
                        op, (unsigned long long)(vm->pc - 1));
                exit(1);
        }
    }
    
    return v_null();
}

/* ─── Cleanup handle tables ─────────────────────────────────────── */
static void cleanup_handles(void) {
    if (!g_handles_initialized) return;
    
    for (int i = 0; i < g_mutex_count; i++) {
        if (g_mutexes[i].active) {
            ovm_mutex_destroy(&g_mutexes[i].mutex);
            g_mutexes[i].active = 0;
        }
    }
    for (int i = 0; i < g_channel_count; i++) {
        if (g_channels[i].active) {
            ovm_mutex_destroy(&g_channels[i].lock);
            ovm_cond_destroy(&g_channels[i].not_empty);
            ovm_cond_destroy(&g_channels[i].not_full);
            free(g_channels[i].buf);
            g_channels[i].active = 0;
        }
    }
    ovm_mutex_destroy(&g_thread_lock);
    ovm_mutex_destroy(&g_mutex_lock);
    ovm_mutex_destroy(&g_atomic_lock);
    ovm_mutex_destroy(&g_channel_lock);
}

/* ─── Main entry ────────────────────────────────────────────────── */
int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "OVM — Omni Virtual Machine v1.1.0\n");
        fprintf(stderr, "Usage: ovm <program.ovm> [args...]\n");
        fprintf(stderr, "       ovm --version\n");
        return 1;
    }
    
    if (strcmp(argv[1], "--version") == 0 || strcmp(argv[1], "-v") == 0) {
        printf("ovm 1.1.0 (Omni Virtual Machine — Standalone, threading support)\n");
        return 0;
    }
    
    /* Initialize threading handle tables */
    init_handle_tables();
    
    /* Read .ovm file */
    FILE *f = fopen(argv[1], "rb");
    if (!f) {
        fprintf(stderr, "OVM: Cannot open '%s'\n", argv[1]);
        return 1;
    }
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    uint8_t *data = malloc((size_t)fsize);
    if (fread(data, 1, (size_t)fsize, f) != (size_t)fsize) {
        fprintf(stderr, "OVM: Read error\n");
        fclose(f);
        free(data);
        return 1;
    }
    fclose(f);
    
    /* Initialize VM */
    OVM vm;
    memset(&vm, 0, sizeof(vm));
    vm.running = 1;
    vm.argc = argc - 2;
    vm.argv = argv + 2;
    
    /* Register native functions */
    register_native(&vm, "print",          native_print);
    register_native(&vm, "println",        native_println);
    register_native(&vm, "to_string",      native_to_string);
    register_native(&vm, "str",            native_to_string);
    register_native(&vm, "int",            native_to_int);
    register_native(&vm, "float",          native_to_float);
    register_native(&vm, "type_of",        native_typeof);
    register_native(&vm, "assert",         native_assert);
    register_native(&vm, "len",            native_len);
    register_native(&vm, "format",         native_format);
    register_native(&vm, "sqrt",           native_sqrt);
    register_native(&vm, "abs",            native_abs);
    register_native(&vm, "pow",            native_pow);
    register_native(&vm, "min",            native_min);
    register_native(&vm, "max",            native_max);
    register_native(&vm, "exit",           native_exit_fn);
    register_native(&vm, "string_concat",  native_string_concat);
    /* Threading intrinsics */
    register_native(&vm, "thread_spawn",    native_thread_spawn);
    register_native(&vm, "thread_join",     native_thread_join);
    register_native(&vm, "thread_sleep_ms", native_thread_sleep_ms);
    register_native(&vm, "thread_yield",    native_thread_yield);
    register_native(&vm, "mutex_new",       native_mutex_new);
    register_native(&vm, "mutex_lock",      native_mutex_lock);
    register_native(&vm, "mutex_unlock",    native_mutex_unlock);
    register_native(&vm, "mutex_try_lock",  native_mutex_try_lock);
    register_native(&vm, "atomic_new",      native_atomic_new);
    register_native(&vm, "atomic_load",     native_atomic_load);
    register_native(&vm, "atomic_store",    native_atomic_store);
    register_native(&vm, "atomic_cas",      native_atomic_cas);
    register_native(&vm, "atomic_fetch_add",native_atomic_fetch_add);
    register_native(&vm, "channel_new",     native_channel_new);
    register_native(&vm, "channel_send",    native_channel_send);
    register_native(&vm, "channel_recv",    native_channel_recv);
    
    /* Load bytecode */
    if (load_ovm(&vm, data, (size_t)fsize) != 0) {
        fprintf(stderr, "OVM: Failed to load '%s'\n", argv[1]);
        free(data);
        return 1;
    }
    
    free(data);
    
    /* Find and call main */
    int main_idx = find_func(&vm, "main");
    if (main_idx < 0) {
        main_idx = (int)vm.entry_point;
    }
    
    if (main_idx >= 0 && (uint32_t)main_idx < vm.func_count) {
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
    free(vm.func_params);
    
    cleanup_handles();
    
    return 0;
}
