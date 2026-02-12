//! Python Buffer Protocol (PEP 3118)
//! 
//! C-ABI compatible structs for zero-copy sharing with NumPy.
//! Implements Py_buffer fill logic and __iter__/__next__ protocol.

use crate::ir::IrType;


#[repr(C)]
pub struct Py_buffer {
    pub buf: *mut u8,
    pub obj: *mut u8, // PyObject*
    pub len: isize,
    pub itemsize: isize,
    pub readonly: i32,
    pub ndim: i32,
    pub format: *mut i8, // "f" for float
    pub shape: *mut isize,
    pub strides: *mut isize,
    pub suboffsets: *mut isize,
    pub internal: *mut u8,
}

impl Py_buffer {
    /// Create a new empty Py_buffer (all null pointers)
    pub fn zeroed() -> Self {
        Self {
            buf: std::ptr::null_mut(),
            obj: std::ptr::null_mut(),
            len: 0,
            itemsize: 0,
            readonly: 0,
            ndim: 0,
            format: std::ptr::null_mut(),
            shape: std::ptr::null_mut(),
            strides: std::ptr::null_mut(),
            suboffsets: std::ptr::null_mut(),
            internal: std::ptr::null_mut(),
        }
    }
    
    /// Fill this buffer view from an Omni Tensor's memory layout.
    /// `data` is the raw pointer to contiguous element storage.
    /// `shape` is the array of dimension sizes (e.g. [batch, height, width, channels]).
    /// `itemsize` is bytes per element (4 for f32, 8 for f64).
    pub fn fill_from_tensor(
        &mut self,
        data: *mut u8,
        shape: &[isize],
        itemsize: isize,
        format: *mut i8,
        readonly: bool,
    ) {
        self.buf = data;
        self.len = shape.iter().product::<isize>() * itemsize;
        self.itemsize = itemsize;
        self.readonly = if readonly { 1 } else { 0 };
        self.ndim = shape.len() as i32;
        self.format = format;
        // shape & strides must outlive this buffer view
        self.shape = shape.as_ptr() as *mut isize;
        // Compute C-contiguous strides
        self.strides = std::ptr::null_mut(); // Will be set by caller
        self.suboffsets = std::ptr::null_mut(); // Not used for simple contiguous
    }
    
    /// Compute C-contiguous (row-major) strides from shape and itemsize
    pub fn compute_strides(shape: &[isize], itemsize: isize) -> Vec<isize> {
        let ndim = shape.len();
        if ndim == 0 {
            return vec![];
        }
        let mut strides = vec![0isize; ndim];
        strides[ndim - 1] = itemsize;
        for i in (0..ndim - 1).rev() {
            strides[i] = strides[i + 1] * shape[i + 1];
        }
        strides
    }
    
    /// Compute Fortran-contiguous (column-major) strides
    pub fn compute_fortran_strides(shape: &[isize], itemsize: isize) -> Vec<isize> {
        let ndim = shape.len();
        if ndim == 0 {
            return vec![];
        }
        let mut strides = vec![0isize; ndim];
        strides[0] = itemsize;
        for i in 1..ndim {
            strides[i] = strides[i - 1] * shape[i - 1];
        }
        strides
    }
}

pub struct BufferProtocol;

impl BufferProtocol {
    /// Map IrType to PEP 3118 struct format character.
    /// See: https://docs.python.org/3/library/struct.html#format-characters
    pub fn get_format_char_for_type(ty: &IrType) -> &'static str {
        match ty {
            IrType::I8 => "b",        // signed char
            IrType::I16 => "h",       // short
            IrType::I32 => "i",       // int
            IrType::I64 => "q",       // long long
            IrType::F32 => "f",       // float
            IrType::F64 => "d",       // double
            IrType::Bool => "?",      // _Bool
            _ => "B",                 // unsigned char fallback
        }
    }
    
    /// Get the byte size for a PEP 3118 format character
    pub fn format_char_size(format: &str) -> usize {
        match format {
            "b" | "B" | "c" | "?" => 1,
            "h" | "H" => 2,
            "i" | "I" | "l" | "L" | "f" => 4,
            "q" | "Q" | "d" | "n" | "N" | "P" => 8,
            _ => 1,
        }
    }
    
    /// Generate C code for __getbuffer__ implementation (bf_getbuffer slot).
    /// This is the function Python calls when someone does `memoryview(omni_obj)` or
    /// `numpy.array(omni_obj)`.
    pub fn generate_getbuffer(type_name: &str, element_type: &IrType, ndim: usize) -> String {
        let format = Self::get_format_char_for_type(element_type);
        let itemsize = Self::format_char_size(format);
        
        format!(r#"
static int {type_name}_getbuffer(PyObject* self, Py_buffer* view, int flags) {{
    OmniTensor* tensor = (OmniTensor*)self;
    
    view->obj = self;
    Py_INCREF(self);
    view->buf = tensor->data;
    view->len = tensor->total_elements * {itemsize};
    view->itemsize = {itemsize};
    view->readonly = 0;
    view->ndim = {ndim};
    view->format = "{format}";
    view->shape = tensor->shape;
    view->strides = tensor->strides;
    view->suboffsets = NULL;
    view->internal = NULL;
    
    return 0;
}}

static void {type_name}_releasebuffer(PyObject* self, Py_buffer* view) {{
    // No-op for contiguous memory owned by Omni runtime
}}

static PyBufferProcs {type_name}_as_buffer = {{
    (getbufferproc){type_name}_getbuffer,
    (releasebufferproc){type_name}_releasebuffer,
}};
"#, type_name = type_name, itemsize = itemsize, ndim = ndim, format = format)
    }

    /// Generate C code for a Python iterator over an Omni collection.
    /// Implements tp_iter and tp_iternext slots.
    pub fn generate_iterator(type_name: &str, element_type: &IrType) -> String {
        let format = Self::get_format_char_for_type(element_type);
        let itemsize = Self::format_char_size(format);
        
        format!(r#"
typedef struct {{
    PyObject_HEAD
    PyObject* source;      // The Omni object being iterated
    Py_ssize_t index;      // Current position
    Py_ssize_t length;     // Total elements
    char* data;            // Pointer to raw data
}} {type_name}Iterator;

static PyObject* {type_name}Iterator_next({type_name}Iterator* self) {{
    if (self->index >= self->length) {{
        PyErr_SetNone(PyExc_StopIteration);
        return NULL;
    }}
    
    void* element_ptr = self->data + (self->index * {itemsize});
    self->index++;
    
    // Convert element to Python object based on format
    {conversion}
}}

static void {type_name}Iterator_dealloc({type_name}Iterator* self) {{
    Py_XDECREF(self->source);
    Py_TYPE(self)->tp_free((PyObject*)self);
}}

static PyObject* {type_name}_iter(PyObject* self) {{
    {type_name}Iterator* it = PyObject_New({type_name}Iterator, &{type_name}IteratorType);
    if (!it) return NULL;
    
    Py_INCREF(self);
    it->source = self;
    it->index = 0;
    it->length = ((OmniTensor*)self)->total_elements;
    it->data = (char*)((OmniTensor*)self)->data;
    
    return (PyObject*)it;
}}

static PyTypeObject {type_name}IteratorType = {{
    PyVarObject_HEAD_INIT(NULL, 0)
    .tp_name = "{type_name}.Iterator",
    .tp_basicsize = sizeof({type_name}Iterator),
    .tp_flags = Py_TPFLAGS_DEFAULT,
    .tp_dealloc = (destructor){type_name}Iterator_dealloc,
    .tp_iternext = (iternextfunc){type_name}Iterator_next,
}};
"#,
            type_name = type_name,
            itemsize = itemsize,
            conversion = match format {
                "f" => "return PyFloat_FromDouble(*(float*)element_ptr);",
                "d" => "return PyFloat_FromDouble(*(double*)element_ptr);",
                "i" | "l" => "return PyLong_FromLong(*(long*)element_ptr);",
                "q" => "return PyLong_FromLongLong(*(long long*)element_ptr);",
                "?" => "return PyBool_FromLong(*(char*)element_ptr);",
                _ => "return PyLong_FromLong(*(unsigned char*)element_ptr);",
            },
        )
    }
}
