// Omni CUDA Kernels
// GPU acceleration for neural network operations

#include <cuda_runtime.h>
#include <cuda_fp16.h>
#include <cublas_v2.h>
#include <curand.h>

// ============================================
// Softmax Kernel
// ============================================
template<typename T>
__global__ void softmax_kernel(
    const T* __restrict__ input,
    T* __restrict__ output,
    int batch_size,
    int dim
) {
    int batch_idx = blockIdx.x;
    int tid = threadIdx.x;
    
    if (batch_idx >= batch_size) return;
    
    const T* row_in = input + batch_idx * dim;
    T* row_out = output + batch_idx * dim;
    
    // Shared memory for reduction
    __shared__ T shared_max;
    __shared__ T shared_sum;
    
    // Find max (reduction)
    T thread_max = -INFINITY;
    for (int i = tid; i < dim; i += blockDim.x) {
        thread_max = fmaxf(thread_max, row_in[i]);
    }
    
    // Warp reduction for max
    for (int offset = 16; offset > 0; offset /= 2) {
        thread_max = fmaxf(thread_max, __shfl_down_sync(0xffffffff, thread_max, offset));
    }
    
    if (tid % 32 == 0) {
        atomicMax((int*)&shared_max, __float_as_int(thread_max));
    }
    __syncthreads();
    
    T max_val = shared_max;
    
    // Compute exp and sum
    T thread_sum = 0.0f;
    for (int i = tid; i < dim; i += blockDim.x) {
        T val = expf(row_in[i] - max_val);
        row_out[i] = val;
        thread_sum += val;
    }
    
    // Warp reduction for sum
    for (int offset = 16; offset > 0; offset /= 2) {
        thread_sum += __shfl_down_sync(0xffffffff, thread_sum, offset);
    }
    
    if (tid % 32 == 0) {
        atomicAdd(&shared_sum, thread_sum);
    }
    __syncthreads();
    
    // Normalize
    T sum_val = shared_sum;
    for (int i = tid; i < dim; i += blockDim.x) {
        row_out[i] /= sum_val;
    }
}

// ============================================
// GELU Activation
// ============================================
__device__ __forceinline__ float gelu(float x) {
    // Approximate GELU: 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
    const float sqrt_2_over_pi = 0.7978845608f;
    const float coef = 0.044715f;
    float inner = sqrt_2_over_pi * (x + coef * x * x * x);
    return 0.5f * x * (1.0f + tanhf(inner));
}

__global__ void gelu_kernel(
    const float* __restrict__ input,
    float* __restrict__ output,
    int n
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        output[idx] = gelu(input[idx]);
    }
}

__global__ void gelu_backward_kernel(
    const float* __restrict__ grad_output,
    const float* __restrict__ input,
    float* __restrict__ grad_input,
    int n
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        float x = input[idx];
        const float sqrt_2_over_pi = 0.7978845608f;
        const float coef = 0.044715f;
        
        float inner = sqrt_2_over_pi * (x + coef * x * x * x);
        float tanh_inner = tanhf(inner);
        float sech2 = 1.0f - tanh_inner * tanh_inner;
        float d_inner = sqrt_2_over_pi * (1.0f + 3.0f * coef * x * x);
        
        float d_gelu = 0.5f * (1.0f + tanh_inner) + 0.5f * x * sech2 * d_inner;
        grad_input[idx] = grad_output[idx] * d_gelu;
    }
}

// ============================================
// Layer Normalization
// ============================================
__global__ void layer_norm_kernel(
    const float* __restrict__ input,
    const float* __restrict__ weight,
    const float* __restrict__ bias,
    float* __restrict__ output,
    int batch_size,
    int hidden_dim,
    float eps
) {
    int batch_idx = blockIdx.x;
    int tid = threadIdx.x;
    
    if (batch_idx >= batch_size) return;
    
    const float* row = input + batch_idx * hidden_dim;
    float* out_row = output + batch_idx * hidden_dim;
    
    // Shared memory for mean and variance
    __shared__ float shared_mean;
    __shared__ float shared_var;
    
    // Compute mean
    float thread_sum = 0.0f;
    for (int i = tid; i < hidden_dim; i += blockDim.x) {
        thread_sum += row[i];
    }
    
    // Warp reduction
    for (int offset = 16; offset > 0; offset /= 2) {
        thread_sum += __shfl_down_sync(0xffffffff, thread_sum, offset);
    }
    
    if (tid % 32 == 0) {
        atomicAdd(&shared_mean, thread_sum);
    }
    __syncthreads();
    
    float mean = shared_mean / hidden_dim;
    
    // Compute variance
    float thread_var = 0.0f;
    for (int i = tid; i < hidden_dim; i += blockDim.x) {
        float diff = row[i] - mean;
        thread_var += diff * diff;
    }
    
    for (int offset = 16; offset > 0; offset /= 2) {
        thread_var += __shfl_down_sync(0xffffffff, thread_var, offset);
    }
    
    if (tid % 32 == 0) {
        atomicAdd(&shared_var, thread_var);
    }
    __syncthreads();
    
    float var = shared_var / hidden_dim;
    float inv_std = rsqrtf(var + eps);
    
    // Normalize and apply scale/bias
    for (int i = tid; i < hidden_dim; i += blockDim.x) {
        float normalized = (row[i] - mean) * inv_std;
        out_row[i] = normalized * weight[i] + bias[i];
    }
}

// ============================================
// Flash Attention (Simplified)
// ============================================
template<int BLOCK_SIZE>
__global__ void flash_attention_kernel(
    const float* __restrict__ Q,
    const float* __restrict__ K,
    const float* __restrict__ V,
    float* __restrict__ O,
    const int seq_len,
    const int head_dim,
    const float scale
) {
    int batch_head = blockIdx.x;
    int q_idx = blockIdx.y * BLOCK_SIZE + threadIdx.x;
    
    if (q_idx >= seq_len) return;
    
    // Pointers for this batch/head
    const float* q = Q + batch_head * seq_len * head_dim + q_idx * head_dim;
    const float* k = K + batch_head * seq_len * head_dim;
    const float* v = V + batch_head * seq_len * head_dim;
    float* o = O + batch_head * seq_len * head_dim + q_idx * head_dim;
    
    // Accumulator
    float acc[128];  // Assume head_dim <= 128
    float max_score = -INFINITY;
    float sum_exp = 0.0f;
    
    for (int d = 0; d < head_dim; d++) {
        acc[d] = 0.0f;
    }
    
    // Process key-value pairs in blocks
    for (int kv_start = 0; kv_start < seq_len; kv_start += BLOCK_SIZE) {
        // Load K block to shared memory
        __shared__ float k_shared[BLOCK_SIZE][128];
        __shared__ float v_shared[BLOCK_SIZE][128];
        
        int kv_idx = kv_start + threadIdx.x;
        if (kv_idx < seq_len) {
            for (int d = 0; d < head_dim; d++) {
                k_shared[threadIdx.x][d] = k[kv_idx * head_dim + d];
                v_shared[threadIdx.x][d] = v[kv_idx * head_dim + d];
            }
        }
        __syncthreads();
        
        // Compute attention scores
        for (int kv_offset = 0; kv_offset < BLOCK_SIZE && kv_start + kv_offset < seq_len; kv_offset++) {
            // Causal mask
            if (kv_start + kv_offset > q_idx) continue;
            
            float score = 0.0f;
            for (int d = 0; d < head_dim; d++) {
                score += q[d] * k_shared[kv_offset][d];
            }
            score *= scale;
            
            // Online softmax update
            float new_max = fmaxf(max_score, score);
            float exp_diff = expf(max_score - new_max);
            float exp_score = expf(score - new_max);
            
            // Rescale accumulator
            for (int d = 0; d < head_dim; d++) {
                acc[d] = acc[d] * exp_diff + exp_score * v_shared[kv_offset][d];
            }
            
            sum_exp = sum_exp * exp_diff + exp_score;
            max_score = new_max;
        }
        __syncthreads();
    }
    
    // Normalize and write output
    for (int d = 0; d < head_dim; d++) {
        o[d] = acc[d] / sum_exp;
    }
}

// ============================================
// RMSNorm (for Llama-style models)
// ============================================
__global__ void rmsnorm_kernel(
    const float* __restrict__ input,
    const float* __restrict__ weight,
    float* __restrict__ output,
    int batch_size,
    int hidden_dim,
    float eps
) {
    int batch_idx = blockIdx.x;
    int tid = threadIdx.x;
    
    const float* row = input + batch_idx * hidden_dim;
    float* out_row = output + batch_idx * hidden_dim;
    
    // Compute RMS
    __shared__ float shared_ss;
    
    float thread_ss = 0.0f;
    for (int i = tid; i < hidden_dim; i += blockDim.x) {
        thread_ss += row[i] * row[i];
    }
    
    for (int offset = 16; offset > 0; offset /= 2) {
        thread_ss += __shfl_down_sync(0xffffffff, thread_ss, offset);
    }
    
    if (tid % 32 == 0) {
        atomicAdd(&shared_ss, thread_ss);
    }
    __syncthreads();
    
    float rms = rsqrtf(shared_ss / hidden_dim + eps);
    
    // Apply normalization and weight
    for (int i = tid; i < hidden_dim; i += blockDim.x) {
        out_row[i] = row[i] * rms * weight[i];
    }
}

// ============================================
// SiLU (Swish) Activation
// ============================================
__global__ void silu_kernel(
    const float* __restrict__ input,
    float* __restrict__ output,
    int n
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        float x = input[idx];
        output[idx] = x / (1.0f + expf(-x));
    }
}

// ============================================
// Rotary Position Embedding
// ============================================
__global__ void rope_kernel(
    float* __restrict__ q,
    float* __restrict__ k,
    const float* __restrict__ cos_cache,
    const float* __restrict__ sin_cache,
    int seq_len,
    int head_dim
) {
    int pos = blockIdx.x;
    int pair_idx = threadIdx.x;
    
    if (pair_idx >= head_dim / 2) return;
    
    float cos_val = cos_cache[pos * head_dim / 2 + pair_idx];
    float sin_val = sin_cache[pos * head_dim / 2 + pair_idx];
    
    int i0 = pair_idx * 2;
    int i1 = pair_idx * 2 + 1;
    
    // Apply rotation to Q
    float q0 = q[pos * head_dim + i0];
    float q1 = q[pos * head_dim + i1];
    q[pos * head_dim + i0] = q0 * cos_val - q1 * sin_val;
    q[pos * head_dim + i1] = q0 * sin_val + q1 * cos_val;
    
    // Apply rotation to K
    float k0 = k[pos * head_dim + i0];
    float k1 = k[pos * head_dim + i1];
    k[pos * head_dim + i0] = k0 * cos_val - k1 * sin_val;
    k[pos * head_dim + i1] = k0 * sin_val + k1 * cos_val;
}

// ============================================
// Cross Entropy Loss
// ============================================
__global__ void cross_entropy_kernel(
    const float* __restrict__ logits,
    const int* __restrict__ targets,
    float* __restrict__ loss,
    int batch_size,
    int vocab_size
) {
    int batch_idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (batch_idx >= batch_size) return;
    
    const float* row = logits + batch_idx * vocab_size;
    int target = targets[batch_idx];
    
    if (target < 0) {  // Ignore index
        loss[batch_idx] = 0.0f;
        return;
    }
    
    // Compute log-softmax and loss
    float max_val = -INFINITY;
    for (int i = 0; i < vocab_size; i++) {
        max_val = fmaxf(max_val, row[i]);
    }
    
    float sum_exp = 0.0f;
    for (int i = 0; i < vocab_size; i++) {
        sum_exp += expf(row[i] - max_val);
    }
    
    float log_softmax = row[target] - max_val - logf(sum_exp);
    loss[batch_idx] = -log_softmax;
}

// ============================================
// Launch wrappers (extern "C" for FFI)
// ============================================
extern "C" {

void launch_softmax(const float* input, float* output, 
                   int batch_size, int dim, cudaStream_t stream) {
    dim3 grid(batch_size);
    dim3 block(256);
    softmax_kernel<float><<<grid, block, 0, stream>>>(input, output, batch_size, dim);
}

void launch_gelu(const float* input, float* output, int n, cudaStream_t stream) {
    int blocks = (n + 255) / 256;
    gelu_kernel<<<blocks, 256, 0, stream>>>(input, output, n);
}

void launch_layer_norm(const float* input, const float* weight, const float* bias,
                      float* output, int batch_size, int hidden_dim, 
                      float eps, cudaStream_t stream) {
    layer_norm_kernel<<<batch_size, 256, 0, stream>>>(
        input, weight, bias, output, batch_size, hidden_dim, eps);
}

void launch_flash_attention(const float* Q, const float* K, const float* V,
                           float* O, int batch_heads, int seq_len, 
                           int head_dim, float scale, cudaStream_t stream) {
    dim3 grid(batch_heads, (seq_len + 31) / 32);
    flash_attention_kernel<32><<<grid, 32, 0, stream>>>(
        Q, K, V, O, seq_len, head_dim, scale);
}

void launch_rmsnorm(const float* input, const float* weight, float* output,
                   int batch_size, int hidden_dim, float eps, cudaStream_t stream) {
    rmsnorm_kernel<<<batch_size, 256, 0, stream>>>(
        input, weight, output, batch_size, hidden_dim, eps);
}

void launch_silu(const float* input, float* output, int n, cudaStream_t stream) {
    int blocks = (n + 255) / 256;
    silu_kernel<<<blocks, 256, 0, stream>>>(input, output, n);
}

void launch_rope(float* q, float* k, const float* cos_cache, const float* sin_cache,
                int seq_len, int head_dim, cudaStream_t stream) {
    rope_kernel<<<seq_len, head_dim / 2, 0, stream>>>(
        q, k, cos_cache, sin_cache, seq_len, head_dim);
}

void launch_cross_entropy(const float* logits, const int* targets, float* loss,
                         int batch_size, int vocab_size, cudaStream_t stream) {
    int blocks = (batch_size + 255) / 256;
    cross_entropy_kernel<<<blocks, 256, 0, stream>>>(
        logits, targets, loss, batch_size, vocab_size);
}

}  // extern "C"
